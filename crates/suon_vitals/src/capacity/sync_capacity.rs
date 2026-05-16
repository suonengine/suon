use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::*;

pub(crate) struct SyncCapacityPlugin;

impl Plugin for SyncCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(sync_capacity_after_capacity_insert)
            .add_observer(sync_capacity_after_capacity_modifiers_insert)
            .add_observer(sync_capacity_after_max_capacity_insert)
            .add_observer(clamp_free_capacity_after_free_capacity_insert);
    }
}

fn sync_capacity_after_capacity_insert(
    event: On<Insert, Capacity>,
    mut capacity: Query<(
        &Capacity,
        &CapacityModifiers,
        &mut MaxCapacity,
        &mut FreeCapacity,
    )>,
) {
    let entity = event.event_target();

    let Ok((capacity, modifiers, mut maximum, mut free_capacity)) = capacity.get_mut(entity) else {
        debug_unreachable!(
            "Capacity sync observer received {entity:?} without required capacity components"
        );
        return;
    };

    sync_capacity(
        entity,
        capacity,
        modifiers,
        &mut maximum,
        &mut free_capacity,
    );
}

fn sync_capacity_after_capacity_modifiers_insert(
    event: On<Insert, CapacityModifiers>,
    mut capacity: Query<(
        &Capacity,
        &CapacityModifiers,
        &mut MaxCapacity,
        &mut FreeCapacity,
    )>,
) {
    let entity = event.event_target();

    let Ok((capacity, modifiers, mut maximum, mut free_capacity)) = capacity.get_mut(entity) else {
        debug_unreachable!(
            "Capacity modifiers sync observer received {entity:?} without required capacity \
             components"
        );
        return;
    };

    sync_capacity(
        entity,
        capacity,
        modifiers,
        &mut maximum,
        &mut free_capacity,
    );
}

fn sync_capacity_after_max_capacity_insert(
    event: On<Insert, MaxCapacity>,
    mut capacity: Query<(
        &Capacity,
        &CapacityModifiers,
        &mut MaxCapacity,
        &mut FreeCapacity,
    )>,
) {
    let entity = event.event_target();

    let Ok((capacity, modifiers, mut maximum, mut free_capacity)) = capacity.get_mut(entity) else {
        debug_unreachable!(
            "Max capacity sync observer received {entity:?} without required capacity components"
        );
        return;
    };

    sync_capacity(
        entity,
        capacity,
        modifiers,
        &mut maximum,
        &mut free_capacity,
    );
}

fn sync_capacity(
    entity: Entity,
    capacity: &Capacity,
    modifiers: &CapacityModifiers,
    maximum: &mut MaxCapacity,
    free_capacity: &mut FreeCapacity,
) {
    let next = MaxCapacity((**capacity as i32).saturating_add(modifiers.total()).max(0) as u32);
    let previous_maximum = *maximum;

    if *maximum != next {
        trace!(
            "Syncing max capacity for {entity:?}: {} -> {}",
            **maximum, *next
        );

        *maximum = next;
    }

    if previous_maximum.0 == 0 {
        **free_capacity = *next;
        return;
    }

    if **free_capacity > *next {
        trace!(
            "Clamping free capacity for {entity:?}: {} -> {}",
            **free_capacity, *next
        );

        **free_capacity = *next;
    }
}

fn clamp_free_capacity_after_free_capacity_insert(
    event: On<Insert, FreeCapacity>,
    mut capacity: Query<(&MaxCapacity, &mut FreeCapacity)>,
) {
    let entity = event.event_target();

    let Ok((maximum, mut free_capacity)) = capacity.get_mut(entity) else {
        debug_unreachable!(
            "Free capacity clamp observer received {entity:?} without MaxCapacity or FreeCapacity"
        );
        return;
    };

    if **free_capacity > **maximum {
        trace!(
            "Clamping free capacity for {entity:?}: {} -> {}",
            **free_capacity, **maximum
        );

        **free_capacity = **maximum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capacity::add_capacity_modifier::{
        AddCapacityModifierIntent, AddCapacityModifierPlugin,
    };

    #[test]
    fn should_initialize_capacity_from_base_and_modifiers() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AddCapacityModifierPlugin,
            SyncCapacityPlugin,
        ));

        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.world_mut().trigger(AddCapacityModifierIntent {
            entity,
            modifier: CapacityModifier::new(50),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            450
        );

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );
    }

    #[test]
    fn should_clamp_free_capacity_when_maximum_decreases() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AddCapacityModifierPlugin,
            SyncCapacityPlugin,
        ));

        let modifier = CapacityModifier::new(100);
        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.world_mut()
            .trigger(AddCapacityModifierIntent { entity, modifier });

        app.update();

        app.world_mut().entity_mut(entity).insert(FreeCapacity(480));

        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert(CapacityModifiers::empty());

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );
    }
}
