use bevy::prelude::*;
use log::{debug, trace};

use super::*;
use crate::health::{Health, heal::Heal};

/// Event emitted when health rises above the flee threshold.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ExitFleeHealth {
    /// Entity whose health crossed the flee threshold upward.
    #[event_target]
    entity: Entity,
}

pub(crate) struct ExitFleeHealthPlugin;

impl Plugin for ExitFleeHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_exit_flee_health_after_heal);
    }
}

fn emit_exit_flee_health_after_heal(
    event: On<Heal>,
    mut commands: Commands,
    vitals: Query<(&Health, &FleeHealth)>,
) {
    let entity = event.event_target();

    let Ok((current, flee_health)) = vitals.get(entity) else {
        debug!("Skipping exit flee health observer for {entity:?}: missing Health or FleeHealth");
        return;
    };

    let exited_flee_health = *event.previous <= **flee_health && **current > **flee_health;
    if exited_flee_health {
        trace!("Emitting exit flee health for {entity:?}");

        commands.trigger(ExitFleeHealth { entity });
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
    struct LastExitFleeHealth {
        entity: Entity,
    }

    fn record(event: On<ExitFleeHealth>, mut commands: Commands) {
        commands.insert_resource(LastExitFleeHealth {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_health_rises_above_threshold() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, ExitFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(20), MaxHealth(100), FleeHealth(30)))
            .id();

        app.world_mut().trigger(HealIntent { entity, amount: 20 });

        app.update();

        assert_eq!(app.world().resource::<LastExitFleeHealth>().entity, entity);
    }

    #[test]
    fn should_emit_when_health_rises_from_exactly_threshold() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, ExitFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(30), MaxHealth(100), FleeHealth(30)))
            .id();

        app.world_mut().trigger(HealIntent { entity, amount: 1 });

        app.update();

        assert_eq!(app.world().resource::<LastExitFleeHealth>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_health_stays_at_or_below_threshold() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, ExitFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(20), MaxHealth(100), FleeHealth(30)))
            .id();

        app.world_mut().trigger(HealIntent { entity, amount: 5 });

        app.update();

        assert!(!app.world().contains_resource::<LastExitFleeHealth>());
    }

    #[test]
    fn should_not_emit_when_entity_has_no_flee_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin, ExitFleeHealthPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(20), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 20 });

        app.update();

        assert!(!app.world().contains_resource::<LastExitFleeHealth>());
    }
}
