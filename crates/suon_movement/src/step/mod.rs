//! Step-based movement systems.
//!
//! Step intents move an entity by one tile according to a [`Direction`], while
//! path advancement consumes queued directions from [`path::StepPath`] using a
//! per-entity [`timer::StepTimer`].

use crate::prelude::*;
use bevy::prelude::*;
use std::time::*;
use suon_chunk::prelude::*;
use suon_position::prelude::*;

pub mod path;
pub mod timer;

/// Plugin that installs step path advancement and step intent handling.
pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, emit_ready_step_path_intents)
            .add_observer(apply_step_intent);
    }
}

#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
/// Intent requesting a one-tile movement for the target entity.
pub struct StepIntent {
    /// Entity that should receive the step.
    #[event_target]
    pub entity: Entity,

    /// Direction to apply to the entity's current position.
    pub to: Direction,
}

/// Reason why a step request was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepError {
    /// The target entity does not have the components required for stepping.
    MissingMovementComponents,
    /// The requested direction does not change the current position.
    TargetMatchesCurrentPosition,
    /// The entity's current coordinate is not mapped to a chunk.
    MissingCurrentChunk,
    /// The requested target coordinate is not mapped to a chunk.
    MissingTargetChunk,
    /// The target coordinate is already occupied on the entity's floor.
    TargetOccupied,
}

/// Event emitted when a step intent cannot be applied.
#[derive(Debug, Clone, Copy, EntityEvent, PartialEq, Eq)]
pub struct StepRejected {
    /// Entity whose step request was rejected.
    #[event_target]
    entity: Entity,

    /// Direction requested by the rejected intent.
    pub to: Direction,

    /// Rejection reason produced by movement validation.
    pub error: StepError,
}

#[derive(EntityEvent)]
/// Event emitted after a step successfully updates the entity position.
///
/// Observers can read the moved entity from the event target.
pub struct Step(Entity);

#[derive(EntityEvent)]
/// Event emitted when a step crosses from one chunk entity to another.
pub struct StepAcrossChunk {
    /// Entity that crossed the chunk boundary.
    #[event_target]
    entity: Entity,

    /// Chunk that previously contained the entity.
    pub from: Entity,

    /// Chunk that now contains the entity after stepping.
    pub to: Entity,
}

/// Advances queued step paths and emits a [`StepIntent`] when an entity is ready to move.
fn emit_ready_step_path_intents(
    mut commands: Commands,
    query: Query<(Entity, &mut StepPath, &mut StepTimer)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut path, mut timer) in query {
        // Only consume the next queued step when the per-entity timer finishes.
        timer.tick(time.delta());
        if !timer.is_finished() {
            continue;
        }

        let Some(target_direction) = path.pop() else {
            continue;
        };

        timer.set_duration(Duration::from_secs(1));
        timer.reset();

        trace!("Emitting step intent for entity {entity:?} toward {target_direction:?}");
        commands.trigger(StepIntent {
            to: target_direction,
            entity,
        });
    }
}

/// Applies a [`StepIntent`] by validating the target tile and emitting success or rejection events.
fn apply_step_intent(
    event: On<StepIntent>,
    mut commands: Commands,
    positions: Query<(&Floor, &Position)>,
    occupancies: Query<&Occupancy>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((floor, position)) = positions.get(entity) else {
        debug!("Rejecting step intent for {entity:?}: missing Floor or Position");
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::MissingMovementComponents,
        });
        return;
    };

    let target_position = *position + event.to;
    if *position == target_position {
        debug!(
            "Rejecting step intent for {entity:?}: direction {:?} keeps position {:?}",
            event.to, position
        );
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::TargetMatchesCurrentPosition,
        });
        return;
    }

    let Some(chunk) = chunks.get(position) else {
        warn!("Rejecting step intent for {entity:?}: current position {position:?} has no chunk");
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::MissingCurrentChunk,
        });
        return;
    };

    let Some(target_chunk) = chunks.get(&target_position) else {
        warn!(
            "Rejecting step intent for {entity:?}: target position {target_position:?} has no \
             chunk"
        );
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::MissingTargetChunk,
        });
        return;
    };

    let Ok(occupancy) = occupancies.get(target_chunk) else {
        warn!(
            "Rejecting step intent for {entity:?}: target chunk {target_chunk:?} has no Occupancy"
        );
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::MissingTargetChunk,
        });
        return;
    };

    if occupancy.contains(floor, &target_position) {
        debug!(
            "Rejecting step intent for {entity:?}: target {:?} on floor {:?} is occupied",
            target_position, floor
        );
        commands.trigger(StepRejected {
            entity,
            to: event.to,
            error: StepError::TargetOccupied,
        });
        return;
    }

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
        .trigger(Step);

    trace!(
        "Applied step for {entity:?}: {:?} -> {:?} on floor {:?}",
        position, target_position, floor
    );

    if chunk != target_chunk {
        trace!("Step for {entity:?} crossed chunk boundary: {chunk:?} -> {target_chunk:?}");
        commands.entity(entity).trigger(|entity| StepAcrossChunk {
            from: chunk,
            to: target_chunk,
            entity,
        });
    }
}

#[cfg(test)]
mod tests {
    use suon_chunk::{CHUNK_SIZE, prelude::*};

    use super::*;
    use std::time::Duration;

    #[test]
    fn should_advance_path_and_reset_timer_when_finished() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.insert_resource(Time::<Fixed>::default());
        app.add_systems(FixedUpdate, emit_ready_step_path_intents);

        let mut path = StepPath::default();
        path.push(Direction::North);

        let entity = app
            .world_mut()
            .spawn((path, StepTimer(Timer::from_seconds(0.5, TimerMode::Once))))
            .id();

        // Progress the fixed clock and execute the system schedule to trigger the tick logic
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(0.6));

        app.world_mut().run_schedule(FixedUpdate);

        let timer = app
            .world()
            .get::<StepTimer>(entity)
            .expect("StepTimer missing");

        assert_eq!(
            timer.duration(),
            Duration::from_secs(1),
            "Timer duration should be 1s"
        );

        assert!(
            !timer.is_finished(),
            "Timer must be in an active state after reset"
        );

        let path = app
            .world()
            .get::<StepPath>(entity)
            .expect("StepPath missing");

        // The system must pop the direction to trigger the event and reset the timer for the next step
        assert!(
            path.is_empty(),
            "The direction should have been consumed from the path"
        );
    }

    #[test]
    fn should_update_position_and_trigger_step_event_on_successful_intent() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_step_intent);

        const START_POSITION: Position = Position {
            x: 0,
            y: CHUNK_SIZE as u16 - 1,
        };
        const EXPECTED_TARGET: Position = Position {
            x: START_POSITION.x,
            y: START_POSITION.y + 1,
        };
        const FLOOR: Floor = Floor { z: 0 };

        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();

        app.insert_resource(Chunks::from_iter([
            (START_POSITION, start_chunk),
            (EXPECTED_TARGET, target_chunk),
        ]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        // Trigger the intent event directly to verify the observer's state transition logic
        app.world_mut().trigger(StepIntent {
            entity,
            to: Direction::North,
        });

        app.update();
        app.update();

        let current_position = app
            .world()
            .get::<Position>(entity)
            .expect("Current position missing");

        assert_eq!(
            *current_position, EXPECTED_TARGET,
            "Actor should be at the target coordinate"
        );

        let previous_position = app
            .world()
            .get::<PreviousPosition>(entity)
            .expect("Previous position missing");

        assert_eq!(
            previous_position.x, START_POSITION.x,
            "The system must record the starting position as the previous one"
        );

        assert!(
            app.world().get::<PreviousFloor>(entity).is_none(),
            "Step should not record PreviousFloor because it does not change floors"
        );

        let at_chunk = app
            .world()
            .get::<AtChunk>(entity)
            .expect("AtChunk reference missing");

        assert_eq!(
            at_chunk.entity(),
            target_chunk,
            "The entity must point to the new chunk entity"
        );
    }

    #[test]
    fn should_prevent_movement_when_target_position_is_occupied() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_step_intent);
        app.add_observer(record_step_rejection);

        const MOVE_DIRECTION: Direction = Direction::East;
        const START_POSITION: Position = Position { x: 5, y: 5 };
        const FLOOR: Floor = Floor { z: 0 };
        let target_position = START_POSITION + MOVE_DIRECTION;

        // Map both coordinates to the same chunk for this collision scenario
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([
            (START_POSITION, chunk),
            (target_position, chunk),
        ]));

        // Spawn an obstructing entity marked as Occupied at the target coordinate
        app.world_mut().spawn((target_position, FLOOR, Occupied));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        // Trigger the intent towards the occupied cell
        app.world_mut().trigger(StepIntent {
            entity,
            to: MOVE_DIRECTION,
        });

        app.update();

        let current_position = app.world().get::<Position>(entity).unwrap();

        // The position must remain identical to the starting position
        assert_eq!(
            *current_position, START_POSITION,
            "Movement must be blocked when a target coordinate contains an Occupied entity"
        );

        let rejection = app
            .world()
            .get_resource::<LastStepRejection>()
            .expect("Step rejection missing");

        assert_eq!(rejection.error, StepError::TargetOccupied);
        assert_eq!(rejection.to, MOVE_DIRECTION);
    }

    #[test]
    fn should_reject_when_moving_to_coordinate_without_registered_chunk() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_step_intent);
        app.add_observer(record_step_rejection);

        const MOVE_DIRECTION: Direction = Direction::North;
        const START_POSITION: Position = Position {
            x: 0,
            y: CHUNK_SIZE as u16 - 1,
        };
        const FLOOR: Floor = Floor { z: 0 };

        // Only provide a chunk for the starting position
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START_POSITION, chunk)]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        app.world_mut().trigger(StepIntent {
            entity,
            to: MOVE_DIRECTION,
        });

        app.update();
        app.update();

        let rejection = app
            .world()
            .get_resource::<LastStepRejection>()
            .expect("Step rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.to, MOVE_DIRECTION);
        assert_eq!(rejection.error, StepError::MissingTargetChunk);
    }

    #[test]
    fn should_not_consume_path_or_reset_timer_before_timer_finishes() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.insert_resource(Time::<Fixed>::default());
        app.add_systems(FixedUpdate, emit_ready_step_path_intents);

        let mut path = StepPath::default();
        path.push(Direction::East);

        let entity = app
            .world_mut()
            .spawn((path, StepTimer(Timer::from_seconds(1.0, TimerMode::Once))))
            .id();

        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(0.25));

        app.world_mut().run_schedule(FixedUpdate);

        let timer = app
            .world()
            .get::<StepTimer>(entity)
            .expect("StepTimer missing");

        let path = app
            .world()
            .get::<StepPath>(entity)
            .expect("StepPath missing");

        assert_eq!(
            path.len(),
            1,
            "The queued step should remain available until the timer finishes"
        );

        assert!(
            !timer.is_finished(),
            "The timer should stay in progress when less than its full duration elapses"
        );
    }

    #[test]
    fn should_ignore_step_intent_when_direction_does_not_change_position() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_step_intent);
        app.add_observer(record_step_rejection);

        const START_POSITION: Position = Position { x: 0, y: 0 };
        const FLOOR: Floor = Floor { z: 0 };

        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START_POSITION, chunk)]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();
        app.world_mut().trigger(StepIntent {
            entity,
            to: Direction::SouthWest,
        });

        app.update();

        assert!(
            app.world().get::<PreviousPosition>(entity).is_none(),
            "A saturating move that keeps the same coordinate should not record PreviousPosition"
        );

        assert_eq!(
            *app.world()
                .get::<Position>(entity)
                .expect("Position missing"),
            START_POSITION,
            "A no-op step direction should leave the position unchanged"
        );

        let rejection = app
            .world()
            .get_resource::<LastStepRejection>()
            .expect("Step rejection missing");

        assert_eq!(rejection.error, StepError::TargetMatchesCurrentPosition);
    }

    #[test]
    fn should_reject_step_intent_when_entity_has_no_position_or_floor() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugins);
        app.add_observer(apply_step_intent);
        app.add_observer(record_step_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(StepIntent {
            entity,
            to: Direction::East,
        });

        app.update();

        assert!(
            app.world().get::<Position>(entity).is_none(),
            "Entities without movement components should be ignored by step intents"
        );

        let rejection = app
            .world()
            .get_resource::<LastStepRejection>()
            .expect("Step rejection missing");

        assert_eq!(rejection.entity, entity);
        assert_eq!(rejection.error, StepError::MissingMovementComponents);
    }

    #[derive(Resource)]
    struct LastStepRejection {
        entity: Entity,
        to: Direction,
        error: StepError,
    }

    /// Stores the last step rejection observed by tests.
    fn record_step_rejection(event: On<StepRejected>, mut commands: Commands) {
        commands.insert_resource(LastStepRejection {
            entity: event.event_target(),
            to: event.to,
            error: event.error,
        });
    }
}
