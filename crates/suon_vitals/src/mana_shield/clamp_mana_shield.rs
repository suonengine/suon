use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::*;

pub(crate) struct ClampManaShieldPlugin;

impl Plugin for ClampManaShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(clamp_after_mana_shield_insert)
            .add_observer(clamp_after_max_mana_shield_insert);
    }
}

fn clamp_after_mana_shield_insert(
    event: On<Insert, ManaShield>,
    mut vitals: Query<(&mut ManaShield, &MaxManaShield)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!(
            "ManaShield clamp observer received {entity:?} without required components"
        );
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping mana_shield for {entity:?}: {} -> {}",
            **current, **maximum
        );

        **current = **maximum;
    }
}

fn clamp_after_max_mana_shield_insert(
    event: On<Insert, MaxManaShield>,
    mut vitals: Query<(&mut ManaShield, &MaxManaShield)>,
) {
    let entity = event.event_target();

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug_unreachable!(
            "MaxManaShield clamp observer received {entity:?} without required components"
        );
        return;
    };

    if **current > **maximum {
        trace!(
            "Clamping mana_shield for {entity:?}: {} -> {}",
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
        app.add_plugins((MinimalPlugins, ClampManaShieldPlugin));

        let entity = app
            .world_mut()
            .spawn((ManaShield(200), MaxManaShield(150)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            150
        );
    }

    #[test]
    fn should_not_clamp_when_current_is_below_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampManaShieldPlugin));

        let entity = app
            .world_mut()
            .spawn((ManaShield(50), MaxManaShield(100)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            50
        );
    }

    #[test]
    fn should_not_clamp_when_current_equals_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampManaShieldPlugin));

        let entity = app
            .world_mut()
            .spawn((ManaShield(100), MaxManaShield(100)))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            100
        );
    }

    #[test]
    fn should_clamp_when_max_mana_shield_is_inserted_below_current() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ClampManaShieldPlugin));

        let entity = app
            .world_mut()
            .spawn((ManaShield(100), MaxManaShield(200)))
            .id();

        app.update();

        app.world_mut().entity_mut(entity).insert(MaxManaShield(50));

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            50
        );
    }
}
