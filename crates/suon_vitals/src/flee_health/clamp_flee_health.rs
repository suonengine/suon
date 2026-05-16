use bevy::prelude::*;
use log::{debug, trace};

use super::*;
use crate::health::MaxHealth;

pub(crate) struct ClampFleeHealthPlugin;

impl Plugin for ClampFleeHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(clamp_after_flee_health_insert)
            .add_observer(clamp_after_max_health_insert);
    }
}

fn clamp_after_flee_health_insert(
    event: On<Insert, FleeHealth>,
    mut vitals: Query<(&mut FleeHealth, &MaxHealth)>,
) {
    let entity = event.event_target();

    let Ok((mut flee_health, maximum)) = vitals.get_mut(entity) else {
        debug!("Skipping flee health clamp for {entity:?}: missing MaxHealth");
        return;
    };

    if **flee_health > **maximum {
        trace!(
            "Clamping flee health for {entity:?}: {} -> {}",
            **flee_health, **maximum
        );

        **flee_health = **maximum;
    }
}

fn clamp_after_max_health_insert(
    event: On<Insert, MaxHealth>,
    mut vitals: Query<(&mut FleeHealth, &MaxHealth)>,
) {
    let entity = event.event_target();

    let Ok((mut flee_health, maximum)) = vitals.get_mut(entity) else {
        debug!("Skipping flee health clamp for {entity:?}: missing FleeHealth");
        return;
    };

    if **flee_health > **maximum {
        trace!(
            "Clamping flee health for {entity:?}: {} -> {}",
            **flee_health, **maximum
        );

        **flee_health = **maximum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clamp_flee_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampFleeHealthPlugin));

        let entity = app
            .world_mut()
            .spawn((FleeHealth(200), MaxHealth(150)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FleeHealth>(entity)
                .expect("FleeHealth should exist"),
            150
        );
    }

    #[test]
    fn should_not_clamp_flee_health_when_below_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampFleeHealthPlugin));

        let entity = app.world_mut().spawn((FleeHealth(25), MaxHealth(100))).id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FleeHealth>(entity)
                .expect("FleeHealth should exist"),
            25
        );
    }

    #[test]
    fn should_clamp_flee_health_when_max_health_is_inserted_below_current() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampFleeHealthPlugin));

        let entity = app
            .world_mut()
            .spawn((FleeHealth(100), MaxHealth(200)))
            .id();

        app.update();

        app.world_mut().entity_mut(entity).insert(MaxHealth(50));

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FleeHealth>(entity)
                .expect("FleeHealth should exist"),
            50
        );
    }
}
