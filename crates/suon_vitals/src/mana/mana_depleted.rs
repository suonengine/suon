use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{consume_mana::ConsumeMana, *};

/// Event emitted when mana reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ManaDepleted {
    /// Entity whose mana reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct ManaDepletedPlugin;

impl Plugin for ManaDepletedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_mana_depleted_after_consume);
    }
}

fn emit_mana_depleted_after_consume(
    event: On<ConsumeMana>,
    mut commands: Commands,
    current: Query<&Mana>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("ManaDepleted observer received {entity:?} without Mana");
        return;
    };

    let mana_depleted = !event.previous.is_zero() && current.is_zero();
    if mana_depleted {
        trace!("Emitting ManaDepleted for {entity:?}");

        commands.trigger(ManaDepleted { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mana::consume_mana::{ConsumeManaIntent, ConsumeManaPlugin};

    #[derive(Resource)]
    struct LastManaDepleted {
        entity: Entity,
    }

    fn record(event: On<ManaDepleted>, mut commands: Commands) {
        commands.insert_resource(LastManaDepleted {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_mana_reaches_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin, ManaDepletedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(50), MaxMana(100))).id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 50 });

        app.update();

        assert_eq!(app.world().resource::<LastManaDepleted>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_mana_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin, ManaDepletedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(100), MaxMana(100))).id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 50 });

        app.update();

        assert!(!app.world().contains_resource::<LastManaDepleted>());
    }
}
