use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{consume_capacity::ConsumeCapacity, *};

/// Event emitted when free capacity reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapacityFull {
    /// Entity whose free capacity reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct CapacityFullPlugin;

impl Plugin for CapacityFullPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_capacity_full_after_consume);
    }
}

fn emit_capacity_full_after_consume(
    event: On<ConsumeCapacity>,
    mut commands: Commands,
    capacity: Query<&FreeCapacity>,
) {
    let entity = event.event_target();

    let Ok(current) = capacity.get(entity) else {
        debug_unreachable!("Capacity full observer received {entity:?} without FreeCapacity");
        return;
    };

    let capacity_full = !event.previous.is_zero() && current.is_zero();
    if capacity_full {
        trace!("Triggering capacity full for {entity:?}");

        commands.trigger(CapacityFull { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capacity::consume_capacity::{ConsumeCapacityIntent, ConsumeCapacityPlugin};

    #[derive(Resource)]
    struct LastCapacityFull {
        entity: Entity,
    }

    fn record_capacity_full(event: On<CapacityFull>, mut commands: Commands) {
        commands.insert_resource(LastCapacityFull {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_capacity_full_after_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin, CapacityFullPlugin));
        app.add_observer(record_capacity_full);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(400)))
            .id();

        app.world_mut().trigger(ConsumeCapacityIntent {
            entity,
            amount: 400,
        });

        app.update();

        assert_eq!(app.world().resource::<LastCapacityFull>().entity, entity);
    }

    #[test]
    fn should_not_emit_capacity_full_when_free_capacity_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin, CapacityFullPlugin));
        app.add_observer(record_capacity_full);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(400)))
            .id();

        app.world_mut().trigger(ConsumeCapacityIntent {
            entity,
            amount: 200,
        });

        app.update();

        assert!(!app.world().contains_resource::<LastCapacityFull>());
    }
}
