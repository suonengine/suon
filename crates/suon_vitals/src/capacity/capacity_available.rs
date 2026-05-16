use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{restore_capacity::RestoreCapacity, *};

/// Event emitted when free capacity rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct CapacityAvailable {
    /// Entity whose free capacity rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct CapacityAvailablePlugin;

impl Plugin for CapacityAvailablePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_capacity_available_after_restore);
    }
}

fn emit_capacity_available_after_restore(
    event: On<RestoreCapacity>,
    mut commands: Commands,
    capacity: Query<&FreeCapacity>,
) {
    let entity = event.event_target();

    let Ok(current) = capacity.get(entity) else {
        debug_unreachable!("Capacity available observer received {entity:?} without FreeCapacity");
        return;
    };

    let capacity_available = event.previous.is_zero() && !current.is_zero();
    if capacity_available {
        trace!("Triggering capacity available for {entity:?}");

        commands.trigger(CapacityAvailable { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capacity::restore_capacity::{RestoreCapacityIntent, RestoreCapacityPlugin};

    #[derive(Resource)]
    struct LastCapacityAvailable {
        entity: Entity,
    }

    fn record_capacity_available(event: On<CapacityAvailable>, mut commands: Commands) {
        commands.insert_resource(LastCapacityAvailable {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_capacity_available_after_restore() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            RestoreCapacityPlugin,
            CapacityAvailablePlugin,
        ));
        app.add_observer(record_capacity_available);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(0)))
            .id();

        app.world_mut()
            .trigger(RestoreCapacityIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastCapacityAvailable>().entity,
            entity
        );
    }

    #[test]
    fn should_not_emit_capacity_available_when_free_capacity_does_not_rise_from_zero() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            RestoreCapacityPlugin,
            CapacityAvailablePlugin,
        ));
        app.add_observer(record_capacity_available);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(200)))
            .id();

        app.world_mut().trigger(RestoreCapacityIntent {
            entity,
            amount: 100,
        });

        app.update();

        assert!(!app.world().contains_resource::<LastCapacityAvailable>());
    }
}
