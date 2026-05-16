use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{restore_mana::RestoreMana, *};

/// Event emitted when mana rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ManaRecovered {
    /// Entity whose mana rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct ManaRecoveredPlugin;

impl Plugin for ManaRecoveredPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_mana_recovered_after_restore);
    }
}

fn emit_mana_recovered_after_restore(
    event: On<RestoreMana>,
    mut commands: Commands,
    current: Query<&Mana>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("ManaRecovered observer received {entity:?} without Mana");
        return;
    };

    let mana_recovered = event.previous.is_zero() && !current.is_zero();
    if mana_recovered {
        trace!("Emitting ManaRecovered for {entity:?}");

        commands.trigger(ManaRecovered { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mana::restore_mana::{RestoreManaIntent, RestoreManaPlugin};

    #[derive(Resource)]
    struct LastManaRecovered {
        entity: Entity,
    }

    fn record(event: On<ManaRecovered>, mut commands: Commands) {
        commands.insert_resource(LastManaRecovered {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_mana_rises_from_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin, ManaRecoveredPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(0), MaxMana(100))).id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 1 });

        app.update();

        assert_eq!(app.world().resource::<LastManaRecovered>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_mana_was_already_above_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin, ManaRecoveredPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(50), MaxMana(100))).id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 25 });

        app.update();

        assert!(!app.world().contains_resource::<LastManaRecovered>());
    }
}
