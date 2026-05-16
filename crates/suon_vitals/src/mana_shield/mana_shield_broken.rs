use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{absorb_mana_shield::AbsorbManaShield, *};

/// Event emitted when mana shield reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ManaShieldBroken {
    /// Entity whose mana shield reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct ManaShieldBrokenPlugin;

impl Plugin for ManaShieldBrokenPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_mana_shield_broken_after_absorb);
    }
}

fn emit_mana_shield_broken_after_absorb(
    event: On<AbsorbManaShield>,
    mut commands: Commands,
    current: Query<&ManaShield>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("ManaShieldBroken observer received {entity:?} without ManaShield");
        return;
    };

    let shield_broken = !event.previous.is_zero() && current.is_zero();
    if shield_broken {
        trace!("Emitting ManaShieldBroken for {entity:?}");

        commands.trigger(ManaShieldBroken { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mana_shield::absorb_mana_shield::{AbsorbManaShieldIntent, AbsorbManaShieldPlugin};

    #[derive(Resource)]
    struct LastManaShieldBroken {
        entity: Entity,
    }

    fn record(event: On<ManaShieldBroken>, mut commands: Commands) {
        commands.insert_resource(LastManaShieldBroken {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_mana_shield_reaches_zero() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AbsorbManaShieldPlugin,
            ManaShieldBrokenPlugin,
        ));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(50), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            app.world().resource::<LastManaShieldBroken>().entity,
            entity
        );
    }

    #[test]
    fn should_not_emit_when_shield_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AbsorbManaShieldPlugin,
            ManaShieldBrokenPlugin,
        ));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(100), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 50 });

        app.update();

        assert!(!app.world().contains_resource::<LastManaShieldBroken>());
    }
}
