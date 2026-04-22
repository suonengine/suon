//! Teleport-based movement systems.
//!
//! Teleport intents relocate an entity directly to a target position when both the
//! current and destination coordinates are registered in [`suon_chunk::chunks::Chunks`].
//! The optional floor payload applies a vertical layer change alongside the
//! horizontal relocation.

use bevy::prelude::*;
use suon_chunk::prelude::*;
use suon_position::prelude::*;

/// Plugin that installs teleport intent handling.
pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_teleport_intent);
    }
}

#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
/// Intent requesting direct relocation of the target entity.
pub struct TeleportIntent {
    /// Entity that should receive the teleport.
    #[event_target]
    pub entity: Entity,

    /// Destination coordinate for the teleport.
    pub to: Position,

    /// Optional destination floor for the teleport.
    pub floor: Option<Floor>,
}

/// Reason why a teleport request was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportError {
    /// The target entity does not have a current position.
    MissingPosition,
    /// The requested destination matches the current position.
    TargetMatchesCurrentPosition,
    /// The entity's current coordinate is not mapped to a chunk.
    MissingCurrentChunk,
    /// The requested destination coordinate is not mapped to a chunk.
    MissingTargetChunk,
}

/// Event emitted when a teleport intent cannot be applied.
#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
pub struct TeleportRejected {
    /// Entity whose teleport request was rejected.
    #[event_target]
    entity: Entity,

    /// Destination requested by the rejected intent.
    pub to: Position,

    /// Floor requested by the rejected intent.
    pub floor: Option<Floor>,

    /// Rejection reason produced by movement validation.
    pub error: TeleportError,
}

#[derive(EntityEvent)]
/// Event emitted after a teleport successfully updates the entity position.
///
/// Observers can read the teleported entity from the event target.
pub struct Teleport(Entity);

#[derive(EntityEvent)]
/// Event emitted when a teleport crosses from one chunk entity to another.
pub struct TeleportAcrossChunk {
    /// Entity that crossed the chunk boundary.
    #[event_target]
    entity: Entity,

    /// Chunk that previously contained the entity.
    pub from: Entity,

    /// Chunk that now contains the entity after teleporting.
    pub to: Entity,
}

/// Applies a [`TeleportIntent`] by validating the destination and emitting success or rejection events.
fn apply_teleport_intent(
    event: On<TeleportIntent>,
    mut commands: Commands,
    positions: Query<(&Position, Option<&Floor>)>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor)) = positions.get(entity) else {
        debug!("Rejecting teleport intent for {entity:?}: missing Position");
        commands.trigger(TeleportRejected {
            entity,
            to: event.to,
            floor: event.floor,
            error: TeleportError::MissingPosition,
        });
        return;
    };

    let target_position = event.to;
    let target_floor = event.floor;
    let position_changes = *position != target_position;
    let floor_changes = target_floor
        .zip(floor.copied())
        .is_some_and(|(target_floor, floor)| target_floor != floor)
        || target_floor.is_some() && floor.is_none();

    if !position_changes && !floor_changes {
        debug!(
            "Rejecting teleport intent for {entity:?}: target position {:?} and floor {:?} do not \
             change state",
            target_position, target_floor
        );
        commands.trigger(TeleportRejected {
            entity,
            to: event.to,
            floor: event.floor,
            error: TeleportError::TargetMatchesCurrentPosition,
        });
        return;
    }

    let Some(chunk) = chunks.get(position) else {
        warn!(
            "Rejecting teleport intent for {entity:?}: current position {position:?} has no chunk"
        );
        commands.trigger(TeleportRejected {
            entity,
            to: event.to,
            floor: event.floor,
            error: TeleportError::MissingCurrentChunk,
        });
        return;
    };

    let Some(target_chunk) = chunks.get(&target_position) else {
        warn!(
            "Rejecting teleport intent for {entity:?}: target position {target_position:?} has no \
             chunk"
        );
        commands.trigger(TeleportRejected {
            entity,
            to: event.to,
            floor: event.floor,
            error: TeleportError::MissingTargetChunk,
        });
        return;
    };

    let mut entity_commands = commands.entity(entity);

    if position_changes {
        entity_commands.insert((
            PreviousPosition {
                x: position.x,
                y: position.y,
            },
            Position {
                x: target_position.x,
                y: target_position.y,
            },
        ));
    }

    if floor_changes {
        if let Some(floor) = floor {
            entity_commands.insert(PreviousFloor { z: floor.z });
        }

        if let Some(target_floor) = target_floor {
            entity_commands.insert(target_floor);
        }
    }

    entity_commands.trigger(Teleport);

    trace!(
        "Applied teleport for {entity:?}: position {:?} -> {:?}, floor {:?} -> {:?}",
        position, target_position, floor, target_floor
    );

    if chunk != target_chunk {
        trace!("Teleport for {entity:?} crossed chunk boundary: {chunk:?} -> {target_chunk:?}");
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

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_teleport_intent);
        app.add_observer(record_teleport_rejection);

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

        const START_FLOOR: Floor = Floor { z: 0 };
        const TARGET_FLOOR: Floor = Floor { z: 2 };

        let entity = app.world_mut().spawn((START, START_FLOOR)).id();

        app.world_mut().trigger(TeleportIntent {
            to: TARGET,
            floor: Some(TARGET_FLOOR),
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

        assert_eq!(
            *app.world().get::<Floor>(entity).expect("Floor missing"),
            TARGET_FLOOR,
            "Teleport should apply the requested floor change"
        );

        assert_eq!(
            *app.world()
                .get::<PreviousFloor>(entity)
                .expect("PreviousFloor missing"),
            PreviousFloor { z: START_FLOOR.z },
            "Teleport should preserve the previous floor for downstream sync"
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

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_teleport_intent);
        app.add_observer(record_teleport_rejection);

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
        app.update();

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "Teleport should no-op when the target matches the current coordinate"
        );

        let rejection = app
            .world()
            .get_resource::<LastTeleportRejection>()
            .expect("Teleport rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.to, POSITION);
        assert_eq!(rejection.error, TeleportError::TargetMatchesCurrentPosition);
    }

    #[test]
    fn should_update_floor_when_teleport_target_position_matches_current_position() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_teleport_intent);

        const POSITION: Position = Position { x: 4, y: 4 };
        const START_FLOOR: Floor = Floor { z: 1 };
        const TARGET_FLOOR: Floor = Floor { z: 3 };

        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(POSITION, chunk)]));

        let entity = app.world_mut().spawn((POSITION, START_FLOOR)).id();

        app.world_mut().trigger(TeleportIntent {
            to: POSITION,
            floor: Some(TARGET_FLOOR),
            entity,
        });

        app.update();

        assert_eq!(
            *app.world()
                .get::<Position>(entity)
                .expect("Position missing"),
            POSITION,
            "Floor-only teleports should leave the position unchanged"
        );

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "Floor-only teleports should not record PreviousPosition"
        );

        assert_eq!(
            *app.world().get::<Floor>(entity).expect("Floor missing"),
            TARGET_FLOOR,
            "Floor-only teleports should apply the requested floor"
        );

        assert_eq!(
            *app.world()
                .get::<PreviousFloor>(entity)
                .expect("PreviousFloor missing"),
            PreviousFloor { z: START_FLOOR.z },
            "Floor-only teleports should record the previous floor"
        );
    }

    #[test]
    fn should_reject_teleport_when_target_position_has_no_registered_chunk() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_teleport_intent);
        app.add_observer(record_teleport_rejection);

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

        let rejection = app
            .world()
            .get_resource::<LastTeleportRejection>()
            .expect("Teleport rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.to, TARGET);
        assert_eq!(rejection.error, TeleportError::MissingTargetChunk);
    }

    #[test]
    fn should_reject_teleport_when_entity_has_no_position() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_teleport_intent);
        app.add_observer(record_teleport_rejection);

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

        let rejection = app
            .world()
            .get_resource::<LastTeleportRejection>()
            .expect("Teleport rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.error, TeleportError::MissingPosition);
    }

    #[derive(Resource)]
    struct LastTeleportRejection {
        entity: Entity,
        to: Position,
        error: TeleportError,
    }

    /// Stores the last teleport rejection observed by tests.
    fn record_teleport_rejection(event: On<TeleportRejected>, mut commands: Commands) {
        commands.insert_resource(LastTeleportRejection {
            entity: event.event_target(),
            to: event.to,
            error: event.error,
        });
    }
}
