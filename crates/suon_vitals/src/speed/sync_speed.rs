use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{speed_changed::SpeedChanged, *};

pub(crate) struct SyncSpeedPlugin;

impl Plugin for SyncSpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(sync_speed_after_base_speed_insert)
            .add_observer(sync_speed_after_speed_modifiers_insert)
            .add_observer(sync_speed_after_speed_insert);
    }
}

fn sync_speed_after_base_speed_insert(
    event: On<Insert, BaseSpeed>,
    mut commands: Commands,
    mut speeds: Query<(&BaseSpeed, &SpeedModifiers, Option<&mut Speed>)>,
) {
    let entity = event.event_target();

    let Ok((base, modifiers, speed)) = speeds.get_mut(entity) else {
        debug_unreachable!(
            "BaseSpeed sync observer received {entity:?} without required speed components"
        );
        return;
    };

    let next = Speed(**base as i32 + modifiers.total());

    let Some(mut current) = speed else {
        trace!("Initializing speed for {entity:?}: {}", *next);

        commands.entity(entity).insert(next);
        return;
    };

    if *current == next {
        return;
    }

    let previous = *current;
    *current = next;

    trace!("Syncing speed for {entity:?}: {} -> {}", *previous, *next);

    commands.trigger(SpeedChanged { entity, previous });
}

fn sync_speed_after_speed_modifiers_insert(
    event: On<Insert, SpeedModifiers>,
    mut commands: Commands,
    mut speeds: Query<(&BaseSpeed, &SpeedModifiers, Option<&mut Speed>)>,
) {
    let entity = event.event_target();

    let Ok((base, modifiers, speed)) = speeds.get_mut(entity) else {
        debug_unreachable!(
            "SpeedModifiers sync observer received {entity:?} without required speed components"
        );
        return;
    };

    let next = Speed(**base as i32 + modifiers.total());

    let Some(mut current) = speed else {
        trace!("Initializing speed for {entity:?}: {}", *next);

        commands.entity(entity).insert(next);
        return;
    };

    if *current == next {
        return;
    }

    let previous = *current;
    *current = next;

    trace!("Syncing speed for {entity:?}: {} -> {}", *previous, *next);

    commands.trigger(SpeedChanged { entity, previous });
}

fn sync_speed_after_speed_insert(
    event: On<Insert, Speed>,
    mut commands: Commands,
    mut speeds: Query<(&BaseSpeed, &SpeedModifiers, &mut Speed)>,
) {
    let entity = event.event_target();

    let Ok((base, modifiers, mut current)) = speeds.get_mut(entity) else {
        debug_unreachable!(
            "Speed sync observer received {entity:?} without required speed components"
        );
        return;
    };

    let next = Speed(**base as i32 + modifiers.total());
    if *current == next {
        return;
    }

    let previous = *current;
    *current = next;

    trace!("Syncing speed for {entity:?}: {} -> {}", *previous, *next);

    commands.trigger(SpeedChanged { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastSpeedChange {
        previous: Speed,
    }

    fn record_speed_changed(event: On<SpeedChanged>, mut commands: Commands) {
        commands.insert_resource(LastSpeedChange {
            previous: event.previous,
        });
    }

    #[test]
    fn should_initialize_speed_from_base_and_modifiers() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SyncSpeedPlugin));

        let entity = app
            .world_mut()
            .spawn((
                BaseSpeed(220),
                SpeedModifiers::new([SpeedModifier::new(40)]),
            ))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            260
        );
    }

    #[test]
    fn should_restore_manual_speed_to_effective_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SyncSpeedPlugin));
        app.add_observer(record_speed_changed);

        let entity = app
            .world_mut()
            .spawn((
                BaseSpeed(220),
                SpeedModifiers::new([SpeedModifier::new(10)]),
                Speed(999),
            ))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            230
        );

        assert_eq!(
            app.world().resource::<LastSpeedChange>().previous,
            Speed(999)
        );
    }

    #[test]
    fn should_allow_negative_effective_speed_from_large_negative_modifier() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SyncSpeedPlugin));

        let entity = app
            .world_mut()
            .spawn((
                BaseSpeed(100),
                SpeedModifiers::new([SpeedModifier::new(-500)]),
            ))
            .id();

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            -400
        );
    }
}
