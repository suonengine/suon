use bevy::prelude::*;
use log::{debug, trace};

use super::*;
use crate::health::{Health, damage::Damage};

/// Event emitted when health drops to or below the flee threshold.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct EnterFleeHealth {
    /// Entity whose health crossed the flee threshold downward.
    #[event_target]
    entity: Entity,
}

pub(crate) struct EnterFleeHealthPlugin;

impl Plugin for EnterFleeHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_enter_flee_health_after_damage);
    }
}

fn emit_enter_flee_health_after_damage(
    event: On<Damage>,
    mut commands: Commands,
    vitals: Query<(&Health, &FleeHealth)>,
) {
    let entity = event.event_target();

    let Ok((current, flee_health)) = vitals.get(entity) else {
        debug!("Skipping enter flee health observer for {entity:?}: missing Health or FleeHealth");
        return;
    };

    let entered_flee_health = *event.previous > **flee_health && **current <= **flee_health;
    if entered_flee_health {
        trace!("Emitting enter flee health for {entity:?}");

        commands.trigger(EnterFleeHealth { entity });
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
    struct LastEnterFleeHealth {
        entity: Entity,
    }

    fn record(event: On<EnterFleeHealth>, mut commands: Commands) {
        commands.insert_resource(LastEnterFleeHealth {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_health_crosses_threshold_downward() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, EnterFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(50), MaxHealth(100), FleeHealth(30)))
            .id();

        app.world_mut().trigger(DamageIntent { entity, amount: 25 });

        app.update();

        assert_eq!(app.world().resource::<LastEnterFleeHealth>().entity, entity);
    }

    #[test]
    fn should_emit_when_health_drops_exactly_to_threshold() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, EnterFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(50), MaxHealth(100), FleeHealth(25)))
            .id();

        app.world_mut().trigger(DamageIntent { entity, amount: 25 });

        app.update();

        assert_eq!(app.world().resource::<LastEnterFleeHealth>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_health_stays_above_threshold() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, EnterFleeHealthPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Health(80), MaxHealth(100), FleeHealth(30)))
            .id();

        app.world_mut().trigger(DamageIntent { entity, amount: 10 });

        app.update();

        assert!(!app.world().contains_resource::<LastEnterFleeHealth>());
    }

    #[test]
    fn should_not_emit_when_entity_has_no_flee_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin, EnterFleeHealthPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(50), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 25 });

        app.update();

        assert!(!app.world().contains_resource::<LastEnterFleeHealth>());
    }
}
