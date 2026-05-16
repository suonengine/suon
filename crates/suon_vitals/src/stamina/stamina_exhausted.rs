use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{consume_stamina::ConsumeStamina, *};

/// Event emitted when stamina reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StaminaExhausted {
    /// Entity whose stamina reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct StaminaExhaustedPlugin;

impl Plugin for StaminaExhaustedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_stamina_exhausted_after_consume);
    }
}

fn emit_stamina_exhausted_after_consume(
    event: On<ConsumeStamina>,
    mut commands: Commands,
    current: Query<&Stamina>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("StaminaExhausted observer received {entity:?} without Stamina");
        return;
    };

    let stamina_exhausted = !event.previous.is_zero() && current.is_zero();
    if stamina_exhausted {
        trace!("Emitting StaminaExhausted for {entity:?}");

        commands.trigger(StaminaExhausted { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stamina::consume_stamina::{ConsumeStaminaIntent, ConsumeStaminaPlugin};

    #[derive(Resource)]
    struct LastStaminaExhausted {
        entity: Entity,
    }

    fn record(event: On<StaminaExhausted>, mut commands: Commands) {
        commands.insert_resource(LastStaminaExhausted {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_stamina_reaches_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin, StaminaExhaustedPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(100), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(100 * 60),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastStaminaExhausted>().entity,
            entity
        );
    }

    #[test]
    fn should_not_emit_when_stamina_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin, StaminaExhaustedPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(100), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(50 * 60),
        });

        app.update();

        assert!(!app.world().contains_resource::<LastStaminaExhausted>());
    }
}
