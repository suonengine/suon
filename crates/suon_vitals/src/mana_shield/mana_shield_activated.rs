use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{restore_mana_shield::RestoreManaShield, *};

/// Event emitted when mana shield rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ManaShieldActivated {
    /// Entity whose mana shield rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct ManaShieldActivatedPlugin;

impl Plugin for ManaShieldActivatedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_mana_shield_activated_after_restore);
    }
}

fn emit_mana_shield_activated_after_restore(
    event: On<RestoreManaShield>,
    mut commands: Commands,
    current: Query<&ManaShield>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("ManaShieldActivated observer received {entity:?} without ManaShield");
        return;
    };

    let shield_activated = event.previous.is_zero() && !current.is_zero();
    if shield_activated {
        trace!("Emitting ManaShieldActivated for {entity:?}");

        commands.trigger(ManaShieldActivated { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mana_shield::restore_mana_shield::{
        RestoreManaShieldIntent, RestoreManaShieldPlugin,
    };

    #[derive(Resource)]
    struct LastManaShieldActivated {
        entity: Entity,
    }

    fn record(event: On<ManaShieldActivated>, mut commands: Commands) {
        commands.insert_resource(LastManaShieldActivated {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_mana_shield_rises_from_zero() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            RestoreManaShieldPlugin,
            ManaShieldActivatedPlugin,
        ));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(0), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastManaShieldActivated>().entity,
            entity
        );
    }

    #[test]
    fn should_not_emit_when_shield_was_already_above_zero() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            RestoreManaShieldPlugin,
            ManaShieldActivatedPlugin,
        ));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(50), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 25 });

        app.update();

        assert!(!app.world().contains_resource::<LastManaShieldActivated>());
    }
}
