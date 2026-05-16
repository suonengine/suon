use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::*;

pub(crate) struct ClampHealthPlugin;

impl Plugin for ClampHealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(clamp_after_health_insert)
            .add_observer(clamp_after_max_health_insert);
    }
}

fn clamp_after_health_insert(
    event: On<Insert, Health>,
    mut vitals: Query<(&mut Health, &MaxHealth)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!("Health clamp observer received {entity:?} without required components");
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping health for {entity:?}: {} -> {}",
            **current, **maximum
        );

        **current = **maximum;
    }
}

fn clamp_after_max_health_insert(
    event: On<Insert, MaxHealth>,
    mut vitals: Query<(&mut Health, &MaxHealth)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!(
            "MaxHealth clamp observer received {entity:?} without required components"
        );
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping health for {entity:?}: {} -> {}",
            **current, **maximum
        );

        **current = **maximum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_clamp_when_current_exceeds_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampHealthPlugin));

        let entity = app.world_mut().spawn((Health(200), MaxHealth(150))).id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            150
        );
    }

    #[test]
    fn should_not_clamp_when_current_is_below_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampHealthPlugin));

        let entity = app.world_mut().spawn((Health(50), MaxHealth(150))).id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            50
        );
    }

    #[test]
    fn should_not_clamp_when_current_equals_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampHealthPlugin));

        let entity = app.world_mut().spawn((Health(100), MaxHealth(100))).id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            100
        );
    }

    #[test]
    fn should_clamp_when_max_health_is_inserted_below_current() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampHealthPlugin));

        let entity = app.world_mut().spawn((Health(100), MaxHealth(150))).id();

        app.update();

        app.world_mut().entity_mut(entity).insert(MaxHealth(50));

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            50
        );
    }
}
