use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use crate::health::{Health, damage::Damage};

/// Event emitted when health reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Die {
    /// Entity whose health reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct DiePlugin;

impl Plugin for DiePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_die_after_damage);
    }
}

fn emit_die_after_damage(event: On<Damage>, mut commands: Commands, health: Query<&Health>) {
    let entity = event.event_target();

    let Ok(current) = health.get(entity) else {
        debug_unreachable!("Die observer received {entity:?} without Health");
        return;
    };

    let just_died = !event.previous.is_zero() && current.is_zero();
    if just_died {
        trace!("Emitting death for {entity:?}");

        commands.trigger(Die { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{
        MaxHealth,
        damage::{DamageIntent, DamagePlugin},
    };

    #[derive(Resource)]
    struct LastDie {
        entity: Entity,
    }

    fn record(event: On<Die>, mut commands: Commands) {
        commands.insert_resource(LastDie {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_die_after_damage() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, DiePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(10), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 20 });

        app.update();

        assert_eq!(app.world().resource::<LastDie>().entity, entity);
    }

    #[test]
    fn should_not_emit_die_when_health_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, DiePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(100), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 50 });

        app.update();

        assert!(!app.world().contains_resource::<LastDie>());
    }
}
