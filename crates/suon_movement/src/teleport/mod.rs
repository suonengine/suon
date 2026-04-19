//! Teleport-based movement systems.
//!
//! Teleport intents relocate an entity directly to a target position when both the
//! current and destination coordinates are registered in [`suon_chunk::chunks::Chunks`].
//! The optional floor payload is currently reserved for future vertical movement and
//! is not applied yet.

use bevy::prelude::*;
use suon_chunk::prelude::*;
use suon_position::prelude::*;

pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_teleport_intent);
    }
}

#[derive(EntityEvent)]
/// Intent requesting direct relocation of the target entity.
pub struct TeleportIntent {
    /// Destination coordinate for the teleport.
    pub to: Position,
    /// Reserved for future floor changes during teleportation.
    pub floor: Option<Floor>,
    #[event_target]
    /// Entity that should receive the teleport.
    pub entity: Entity,
}

#[derive(EntityEvent)]
/// Event emitted after a teleport successfully updates the entity position.
pub struct Teleport(Entity);

#[derive(EntityEvent)]
/// Event emitted when a teleport crosses from one chunk entity to another.
pub struct TeleportAcrossChunk {
    /// Chunk that previously contained the entity.
    pub from: Entity,
    /// Chunk that now contains the entity after teleporting.
    pub to: Entity,
    #[event_target]
    entity: Entity,
}

fn on_teleport_intent(
    event: On<TeleportIntent>,
    mut commands: Commands,
    positions: Query<&Position>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok(position) = positions.get(entity) else {
        return;
    };

    let target_position = event.to;
    if position == &target_position {
        return;
    }

    let Some(chunk) = chunks.get(position) else {
        return;
    };

    let Some(target_chunk) = chunks.get(&target_position) else {
        return;
    };

    commands
        .entity(entity)
        .insert((
            PreviousPosition {
                x: position.x,
                y: position.y,
            },
            Position {
                x: target_position.x,
                y: target_position.y,
            },
        ))
        .trigger(Teleport);

    if chunk != target_chunk {
        commands
            .entity(entity)
            .trigger(|entity| TeleportAcrossChunk {
                from: chunk,
                to: target_chunk,
                entity,
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suon_chunk::{self, CHUNK_SIZE};

    #[test]
    fn should_update_position_on_successful_teleport() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_teleport_intent);

        const START: Position = Position { x: 0, y: 0 };
        const TARGET: Position = Position {
            x: CHUNK_SIZE as u16,
            y: 0,
        };

        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([
            (START, start_chunk),
            (TARGET, target_chunk),
        ]));

        let entity = app.world_mut().spawn(START).id();

        app.world_mut().trigger(TeleportIntent {
            to: TARGET,
            floor: Some(Floor { z: 2 }),
            entity,
        });

        app.update();
        app.update();

        assert_eq!(
            *app.world()
                .get::<Position>(entity)
                .expect("Position missing"),
            TARGET,
            "Teleport should move the entity directly to the requested target coordinate"
        );

        let previous = app
            .world()
            .get::<PreviousPosition>(entity)
            .expect("PreviousPosition missing");

        assert_eq!(
            (previous.x, previous.y),
            (START.x, START.y),
            "Teleport should preserve the previous coordinate for downstream sync"
        );

        let at_chunk = app.world().get::<AtChunk>(entity).expect("AtChunk missing");

        assert_eq!(
            at_chunk.entity(),
            target_chunk,
            "Teleport should allow downstream chunk sync to point at the destination chunk"
        );
    }

    #[test]
    fn should_ignore_teleport_when_target_matches_current_position() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_teleport_intent);

        const POSITION: Position = Position { x: 4, y: 4 };
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(POSITION, chunk)]));

        let entity = app.world_mut().spawn(POSITION).id();

        app.world_mut().trigger(TeleportIntent {
            to: POSITION,
            floor: None,
            entity,
        });

        app.update();

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "Teleport should no-op when the target matches the current coordinate"
        );
    }

    #[test]
    fn should_ignore_teleport_when_target_position_has_no_registered_chunk() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_teleport_intent);

        const START: Position = Position { x: 1, y: 1 };
        const TARGET: Position = Position { x: 99, y: 99 };
        let entity_chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START, entity_chunk)]));

        let entity = app.world_mut().spawn(START).id();

        app.world_mut().trigger(TeleportIntent {
            to: TARGET,
            floor: None,
            entity,
        });

        app.update();

        assert_eq!(
            *app.world()
                .get::<Position>(entity)
                .expect("Position missing"),
            START,
            "Teleport should not move entities into unmapped chunk space"
        );

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "Rejected teleports should not record a previous position"
        );
    }

    #[test]
    fn should_ignore_teleport_when_entity_has_no_position() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_teleport_intent);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(TeleportIntent {
            to: Position { x: 2, y: 2 },
            floor: None,
            entity,
        });

        app.update();

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "Teleport intents should no-op when the target entity has no current position"
        );
    }
}
