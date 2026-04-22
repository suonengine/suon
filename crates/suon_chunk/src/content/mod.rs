//! Entity-to-chunk relationships.
//!
//! [`crate::content::AtChunk`] is derived from
//! [`suon_position::prelude::Position`] by the crate runtime and mirrored back
//! into [`crate::content::Content`] through Bevy relationships.

use bevy::prelude::*;
use suon_position::prelude::*;

/// Plugin responsible for deriving [`AtChunk`] relationships from positions.
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        debug!("Installing chunk content observers");
        app.add_observer(update_at_chunk_after_position_change);
    }
}

#[derive(Component, Deref, Debug)]
#[relationship_target(relationship = AtChunk, linked_spawn)]
/// Stores the entities currently linked to a chunk through [`AtChunk`].
pub struct Content(Vec<Entity>);

#[derive(Component, Deref, Debug)]
#[relationship(relationship_target = Content)]
#[component(immutable)]
/// Relationship component pointing an entity to the chunk that contains it.
pub struct AtChunk(#[entities] Entity);

impl AtChunk {
    /// Creates a chunk relationship pointing at the provided chunk entity.
    ///
    /// This constructor stays crate-visible so chunk ownership continues to be
    /// derived from [`suon_position::prelude::Position`] rather than assigned
    /// manually by other crates.
    pub(crate) fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Returns the chunk entity currently linked to this component.
    ///
    /// # Examples
    /// ```no_run
    /// use bevy::prelude::*;
    /// use suon_chunk::prelude::*;
    /// use suon_position::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.add_plugins(MinimalPlugins);
    /// app.add_plugins(ChunkPlugins);
    ///
    /// let chunk = app.world_mut().spawn(Chunk).id();
    /// app.insert_resource(Chunks::from_iter([(Position { x: 1, y: 1 }, chunk)]));
    ///
    /// let entity = app.world_mut().spawn(Position { x: 1, y: 1 }).id();
    /// app.update();
    ///
    /// let at_chunk = app.world().get::<AtChunk>(entity).unwrap();
    /// assert_eq!(at_chunk.entity(), chunk);
    /// ```
    pub fn entity(&self) -> Entity {
        self.0
    }
}

/// Reason why an [`AtChunk`] update could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtChunkUpdateError {
    /// The entity no longer has the inserted position when the observer runs.
    MissingPosition,
    /// The current position does not resolve to a registered chunk.
    MissingRegisteredChunk,
}

/// Event emitted when [`AtChunk`] cannot be derived from an entity position.
#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
pub struct AtChunkUpdateRejected {
    /// Entity whose chunk relationship could not be updated.
    #[event_target]
    entity: Entity,

    /// Position that failed chunk ownership lookup, when available.
    pub position: Option<Position>,

    /// Rejection reason produced by chunk ownership validation.
    pub error: AtChunkUpdateError,
}

/// Derives [`AtChunk`] whenever [`suon_position::prelude::Position`] is inserted
/// or replaced.
///
/// If the entity's position no longer resolves to a registered chunk, the stale
/// [`AtChunk`] component is removed and [`AtChunkUpdateRejected`] is emitted.
pub(crate) fn update_at_chunk_after_position_change(
    event: On<Insert, Position>,
    mut commands: Commands,
    entities: Query<(&Position, Option<&AtChunk>)>,
    chunks: Res<crate::chunks::Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, at_chunk)) = entities.get(entity) else {
        debug!("Rejecting AtChunk update for {entity:?}: missing Position");

        commands.trigger(AtChunkUpdateRejected {
            entity,
            position: None,
            error: AtChunkUpdateError::MissingPosition,
        });
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        warn!("Rejecting AtChunk update for {entity:?}: position {position:?} has no chunk");

        commands.entity(entity).remove::<AtChunk>();

        commands.trigger(AtChunkUpdateRejected {
            entity,
            position: Some(*position),
            error: AtChunkUpdateError::MissingRegisteredChunk,
        });
        return;
    };

    if at_chunk.is_some_and(|current_chunk| current_chunk.entity() == chunk) {
        trace!("AtChunk for {entity:?} already points to chunk {chunk:?}");
        return;
    }

    trace!("Updating AtChunk for {entity:?} at position {position:?} to chunk {chunk:?}");

    commands.entity(entity).insert(AtChunk::new(chunk));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, ChunkPlugins, chunks::Chunks};

    #[derive(Resource)]
    struct LastAtChunkRejection {
        entity: Entity,
        position: Option<Position>,
        error: AtChunkUpdateError,
    }

    /// Stores the last AtChunk update rejection observed by tests.
    fn record_at_chunk_rejection(event: On<AtChunkUpdateRejected>, mut commands: Commands) {
        commands.insert_resource(LastAtChunkRejection {
            entity: event.event_target(),
            position: event.position,
            error: event.error,
        });
    }

    #[test]
    fn should_reject_at_chunk_update_when_position_has_no_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);
        app.add_observer(record_at_chunk_rejection);

        let old_chunk = app.world_mut().spawn(Chunk).id();
        let entity = app
            .world_mut()
            .spawn((Position { x: 1, y: 1 }, AtChunk::new(old_chunk)))
            .id();

        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert(Position { x: 99, y: 99 });

        app.update();

        assert!(
            app.world().get::<AtChunk>(entity).is_none(),
            "Entities should lose AtChunk when their position no longer resolves to a chunk"
        );

        let rejection = app
            .world()
            .get_resource::<LastAtChunkRejection>()
            .expect("AtChunk rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.position, Some(Position { x: 99, y: 99 }));
        assert_eq!(rejection.error, AtChunkUpdateError::MissingRegisteredChunk);
    }

    #[test]
    fn should_replace_at_chunk_when_position_moves_to_another_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let first_chunk = app.world_mut().spawn(Chunk).id();
        let second_chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([
            (Position { x: 1, y: 1 }, first_chunk),
            (Position { x: 8, y: 1 }, second_chunk),
        ]));

        let entity = app.world_mut().spawn(Position { x: 1, y: 1 }).id();

        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert(Position { x: 8, y: 1 });

        app.update();

        let at_chunk = app
            .world()
            .get::<AtChunk>(entity)
            .expect("Position sync should keep AtChunk in sync with the new chunk");

        assert_eq!(
            at_chunk.entity(),
            second_chunk,
            "AtChunk should be updated when an entity moves to another registered chunk"
        );
    }
}
