use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use crate::health::{Health, heal::Heal};

/// Event emitted when health rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Revive {
    /// Entity whose health rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct RevivePlugin;

impl Plugin for RevivePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_revive_after_heal);
    }
}

fn emit_revive_after_heal(event: On<Heal>, mut commands: Commands, health: Query<&Health>) {
    let entity = event.event_target();

    let Ok(current) = health.get(entity) else {
        debug_unreachable!("Revive observer received {entity:?} without Health");
        return;
    };

    let just_revived = event.previous.is_zero() && !current.is_zero();
    if just_revived {
        trace!("Emitting revive for {entity:?}");

        commands.trigger(Revive { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{
        MaxHealth,
        heal::{HealIntent, HealPlugin},
    };

    #[derive(Resource)]
    struct LastRevive {
        entity: Entity,
    }

    fn record(event: On<Revive>, mut commands: Commands) {
        commands.insert_resource(LastRevive {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_revive_after_heal() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, RevivePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(0), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 20 });

        app.update();

        assert_eq!(app.world().resource::<LastRevive>().entity, entity);
    }

    #[test]
    fn should_not_emit_revive_when_health_does_not_rise_from_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, RevivePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(50), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 25 });

        app.update();

        assert!(!app.world().contains_resource::<LastRevive>());
    }
}
