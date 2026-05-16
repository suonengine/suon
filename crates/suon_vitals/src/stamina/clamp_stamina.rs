use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::*;

pub(crate) struct ClampStaminaPlugin;

impl Plugin for ClampStaminaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(clamp_after_stamina_insert)
            .add_observer(clamp_after_max_stamina_insert);
    }
}

fn clamp_after_stamina_insert(
    event: On<Insert, Stamina>,
    mut vitals: Query<(&mut Stamina, &MaxStamina)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!(
            "Stamina clamp observer received {entity:?} without required components"
        );
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping stamina for {entity:?}: {:?} -> {:?}",
            **current, **maximum
        );

        **current = **maximum;
    }
}

fn clamp_after_max_stamina_insert(
    event: On<Insert, MaxStamina>,
    mut vitals: Query<(&mut Stamina, &MaxStamina)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!(
            "MaxStamina clamp observer received {entity:?} without required components"
        );
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping stamina for {entity:?}: {:?} -> {:?}",
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
        app.add_plugins((MinimalPlugins, ClampStaminaPlugin));

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(200), MaxStamina::from_minutes(150)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(150 * 60)
        );
    }

    #[test]
    fn should_not_clamp_when_current_is_below_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampStaminaPlugin));

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(50), MaxStamina::from_minutes(100)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(50 * 60)
        );
    }

    #[test]
    fn should_not_clamp_when_current_equals_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampStaminaPlugin));

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(100), MaxStamina::from_minutes(100)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(100 * 60)
        );
    }

    #[test]
    fn should_clamp_when_max_stamina_is_inserted_below_current() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampStaminaPlugin));

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(100), MaxStamina::from_minutes(200)))
            .id();

        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert(MaxStamina::from_minutes(50));

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(50 * 60)
        );
    }
}
