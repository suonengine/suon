use bevy::prelude::*;
use log::debug;

use super::*;

/// Reason why an add capacity modifier intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddCapacityModifierIntentError {
    /// The target entity does not have [`Capacity`].
    MissingCapacity,

    /// The target entity does not have [`CapacityModifiers`].
    MissingCapacityModifiers,
}

/// Intent requesting a capacity modifier to be added or replaced for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AddCapacityModifierIntent {
    /// Entity whose capacity modifier should be added or replaced.
    #[event_target]
    pub entity: Entity,

    /// Modifier to add or replace.
    pub modifier: CapacityModifier,
}

/// Event emitted when an add capacity modifier intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AddCapacityModifierRejected {
    /// Entity whose add capacity modifier intent was rejected.
    #[event_target]
    entity: Entity,

    /// Rejection reason.
    pub error: AddCapacityModifierIntentError,
}

pub(crate) struct AddCapacityModifierPlugin;

impl Plugin for AddCapacityModifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_add_capacity_modifier_intent);
    }
}

fn apply_add_capacity_modifier_intent(
    event: On<AddCapacityModifierIntent>,
    mut commands: Commands,
    capacity: Query<(&Capacity, Option<&CapacityModifiers>)>,
) {
    let entity = event.event_target();

    let Ok((.., modifiers)) = capacity.get(entity) else {
        debug!("Rejecting add capacity modifier intent for {entity:?}: missing Capacity");

        commands.trigger(AddCapacityModifierRejected {
            entity,
            error: AddCapacityModifierIntentError::MissingCapacity,
        });
        return;
    };

    let Some(modifiers) = modifiers else {
        debug!("Rejecting add capacity modifier intent for {entity:?}: missing CapacityModifiers");

        commands.trigger(AddCapacityModifierRejected {
            entity,
            error: AddCapacityModifierIntentError::MissingCapacityModifiers,
        });
        return;
    };

    commands
        .entity(entity)
        .insert(modifiers.with_added(event.modifier));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capacity::sync_capacity::SyncCapacityPlugin;

    #[test]
    fn should_add_capacity_modifier() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AddCapacityModifierPlugin,
            SyncCapacityPlugin,
        ));

        let modifier = CapacityModifier::new(50);
        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.update();

        app.world_mut()
            .trigger(AddCapacityModifierIntent { entity, modifier });

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

    #[derive(Resource)]
    struct LastAddCapacityModifierRejection {
        error: AddCapacityModifierIntentError,
    }

    fn record_add_rejection(event: On<AddCapacityModifierRejected>, mut commands: Commands) {
        commands.insert_resource(LastAddCapacityModifierRejection { error: event.error });
    }

    #[test]
    fn should_reject_add_capacity_modifier_without_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddCapacityModifierPlugin));
        app.add_observer(record_add_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(AddCapacityModifierIntent {
            entity,
            modifier: CapacityModifier::new(50),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastAddCapacityModifierRejection>()
                .error,
            AddCapacityModifierIntentError::MissingCapacity
        );
    }

    #[test]
    fn should_replace_capacity_modifier_with_same_id() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AddCapacityModifierPlugin,
            SyncCapacityPlugin,
        ));

        let modifier = CapacityModifier::new(50);
        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.world_mut()
            .trigger(AddCapacityModifierIntent { entity, modifier });

        app.update();

        let replacement = CapacityModifier {
            id: modifier.id,
            value: 200,
        };

        app.world_mut().trigger(AddCapacityModifierIntent {
            entity,
            modifier: replacement,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            600
        );

        assert_eq!(
            app.world()
                .get::<CapacityModifiers>(entity)
                .expect("CapacityModifiers should exist")
                .iter()
                .count(),
            1
        );
    }
}
