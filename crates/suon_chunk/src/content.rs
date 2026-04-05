//! Entity-to-chunk relationships.
//!
//! [`crate::content::AtChunk`] is derived from
//! [`suon_position::position::Position`] by the crate runtime and mirrored back
//! into [`crate::content::Content`] through Bevy relationships.

use bevy::prelude::*;
use suon_position::position::Position;

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
    /// derived from [`suon_position::position::Position`] rather than assigned
    /// manually by other crates.
    pub(crate) fn new(entity: Entity) -> Self {
        Self(entity)
    }

    /// Returns the chunk entity currently linked to this component.
    ///
    /// # Examples
    /// ```no_run
    /// use bevy::prelude::*;
    /// use suon_chunk::{Chunk, ChunkPlugin, chunks::Chunks, content::AtChunk};
    /// use suon_position::position::Position;
    ///
    /// let mut app = App::new();
    /// app.add_plugins(MinimalPlugins);
    /// app.add_plugins(ChunkPlugin);
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

/// Derives [`AtChunk`] whenever [`suon_position::position::Position`] is inserted
/// or replaced.
///
/// If the entity's position no longer resolves to a registered chunk, the stale
/// [`AtChunk`] component is removed.
pub(crate) fn sync_at_chunk_from_position(
    event: On<Insert, Position>,
    mut commands: Commands,
    entities: Query<(&Position, Option<&AtChunk>)>,
    chunks: Res<crate::chunks::Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, at_chunk)) = entities.get(entity) else {
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        commands.entity(entity).remove::<AtChunk>();
        return;
    };

    if at_chunk.is_some_and(|current_chunk| current_chunk.entity() == chunk) {
        return;
    }

    commands.entity(entity).insert(AtChunk::new(chunk));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, ChunkPlugin, chunks::Chunks};

    #[test]
    fn should_remove_stale_at_chunk_when_position_has_no_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

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
    }

    #[test]
    fn should_replace_at_chunk_when_position_moves_to_another_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

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
