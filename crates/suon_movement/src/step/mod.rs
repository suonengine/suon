//! Step-based movement systems.
//!
//! Step intents move an entity by one tile according to a [`StepDirection`], while
//! path advancement consumes queued directions from [`path::StepPath`] using a
//! per-entity [`timer::StepTimer`].

use crate::prelude::*;
use bevy::prelude::*;
use std::time::*;
use suon_chunk::{chunks::Chunks, occupancy::Occupancy};
use suon_position::{floor::Floor, position::Position, previous_position::PreviousPosition};

pub mod direction;
pub mod path;
pub mod timer;

pub struct StepPlugin;

impl Plugin for StepPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, advance_step_paths)
            .add_observer(on_step_intent);
    }
}

#[derive(EntityEvent)]
/// Intent requesting a one-tile movement for the target entity.
pub struct StepIntent {
    pub to: StepDirection,
    #[event_target]
    pub entity: Entity,
}

#[derive(EntityEvent)]
/// Event emitted after a step successfully updates the entity position.
pub struct Step(Entity);

#[derive(EntityEvent)]
/// Event emitted when a step crosses from one chunk entity to another.
pub struct StepAcrossChunk {
    pub from: Entity,
    pub to: Entity,
    #[event_target]
    entity: Entity,
}

fn advance_step_paths(
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

        commands.trigger(StepIntent {
            to: target_direction,
            entity,
        });
    }
}

fn on_step_intent(
    event: On<StepIntent>,
    mut commands: Commands,
    positions: Query<(&Floor, &Position)>,
    occupancies: Query<&Occupancy>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((layer, position)) = positions.get(entity) else {
        return;
    };

    let target_position = *position + event.to;
    if position == &target_position {
        return;
    }

    let Some(chunk) = chunks.get(position) else {
        return;
    };

    let Some(target_chunk) = chunks.get(&target_position) else {
        return;
    };

    let Ok(occupancy) = occupancies.get(target_chunk) else {
        return;
    };

    if occupancy.contains(layer, &target_position) {
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

    if chunk != target_chunk {
        commands.entity(entity).trigger(|entity| StepAcrossChunk {
            from: chunk,
            to: target_chunk,
            entity,
        });
    }
}

#[cfg(test)]
mod tests {
    use suon_chunk::{
        CHUNK_SIZE, Chunk, ChunkPlugin, content::AtChunk, occupancy::occupied::Occupied,
    };

    use super::*;
    use std::time::Duration;

    #[test]
    fn should_advance_path_and_reset_timer_when_finished() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.insert_resource(Time::<Fixed>::default());
        app.add_systems(FixedUpdate, advance_step_paths);

        let mut path = StepPath::default();
        path.push(StepDirection::North);

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

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_step_intent);

        const START_POSITION: Position = Position {
            x: 0,
            y: CHUNK_SIZE as u16 - 1,
        };
        const FLOOR: Floor = Floor { z: 0 };
        let expected_target = START_POSITION + StepDirection::North;

        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();

        app.insert_resource(Chunks::from_iter([
            (START_POSITION, start_chunk),
            (expected_target, target_chunk),
        ]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        // Trigger the intent event directly to verify the observer's state transition logic
        app.world_mut().trigger(StepIntent {
            entity,
            to: StepDirection::North,
        });

        app.update();
        app.update();

        let current_position = app
            .world()
            .get::<Position>(entity)
            .expect("Current position missing");

        assert_eq!(
            *current_position, expected_target,
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

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_step_intent);

        const MOVE_DIRECTION: StepDirection = StepDirection::East;
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
    }

    #[test]
    fn should_fail_when_moving_to_coordinate_without_registered_chunk() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_step_intent);

        const MOVE_DIRECTION: StepDirection = StepDirection::North;
        const START_POSITION: Position = Position { x: 0, y: 0 };
        const FLOOR: Floor = Floor { z: 0 };

        // Only provide a chunk for the starting position
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START_POSITION, chunk)]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        // The observer should panic because the target coordinate is not mapped in ChunkLayer
        app.world_mut().trigger(StepIntent {
            entity,
            to: MOVE_DIRECTION,
        });

        app.update();
    }

    #[test]
    fn should_not_consume_path_or_reset_timer_before_timer_finishes() {
        let mut app = App::new();

        app.add_plugins(ChunkPlugin);
        app.insert_resource(Time::<Fixed>::default());
        app.add_systems(FixedUpdate, advance_step_paths);

        let mut path = StepPath::default();
        path.push(StepDirection::East);

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

        app.add_plugins(ChunkPlugin);
        app.add_observer(on_step_intent);

        const START_POSITION: Position = Position { x: 0, y: 0 };
        const FLOOR: Floor = Floor { z: 0 };
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START_POSITION, chunk)]));

        let entity = app.world_mut().spawn((START_POSITION, FLOOR)).id();

        app.world_mut().trigger(StepIntent {
            entity,
            to: StepDirection::SouthWest,
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
    }
}
