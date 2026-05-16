use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{restore_stamina::RestoreStamina, *};

/// Event emitted when stamina rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StaminaRecovered {
    /// Entity whose stamina rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct StaminaRecoveredPlugin;

impl Plugin for StaminaRecoveredPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_stamina_recovered_after_restore);
    }
}

fn emit_stamina_recovered_after_restore(
    event: On<RestoreStamina>,
    mut commands: Commands,
    current: Query<&Stamina>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("StaminaRecovered observer received {entity:?} without Stamina");
        return;
    };

    let stamina_recovered = event.previous.is_zero() && !current.is_zero();
    if stamina_recovered {
        trace!("Emitting StaminaRecovered for {entity:?}");

        commands.trigger(StaminaRecovered { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamina::restore_stamina::{RestoreStaminaIntent, RestoreStaminaPlugin};

    #[derive(Resource)]
    struct LastStaminaRecovered {
        entity: Entity,
    }

    fn record(event: On<StaminaRecovered>, mut commands: Commands) {
        commands.insert_resource(LastStaminaRecovered {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_stamina_rises_from_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin, StaminaRecoveredPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina(Duration::ZERO), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(1),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastStaminaRecovered>().entity,
            entity
        );
    }

    #[test]
    fn should_not_emit_when_stamina_was_already_above_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin, StaminaRecoveredPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(50), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(25 * 60),
        });

        app.update();

        assert!(!app.world().contains_resource::<LastStaminaRecovered>());
    }
}
