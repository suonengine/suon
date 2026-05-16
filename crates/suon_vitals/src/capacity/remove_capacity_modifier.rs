use bevy::prelude::*;
use log::debug;

use super::*;

/// Reason why a remove capacity modifier intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveCapacityModifierIntentError {
    /// The target entity does not have [`Capacity`].
    MissingCapacity,
    /// The target entity does not have [`CapacityModifiers`].
    MissingCapacityModifiers,
    /// The target entity does not have a modifier with the requested id.
    MissingModifier,
}

/// Intent requesting a capacity modifier removal for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RemoveCapacityModifierIntent {
    /// Entity whose capacity modifier should be removed.
    #[event_target]
    pub entity: Entity,

    /// Modifier id to remove.
    pub id: CapacityModifierId,
}

/// Event emitted when a remove capacity modifier intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RemoveCapacityModifierRejected {
    /// Entity whose remove capacity modifier intent was rejected.
    #[event_target]
    entity: Entity,

    /// Rejection reason.
    pub error: RemoveCapacityModifierIntentError,
}

pub(crate) struct RemoveCapacityModifierPlugin;

impl Plugin for RemoveCapacityModifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_remove_capacity_modifier_intent);
    }
}

fn apply_remove_capacity_modifier_intent(
    event: On<RemoveCapacityModifierIntent>,
    mut commands: Commands,
    capacity: Query<(&Capacity, Option<&CapacityModifiers>)>,
) {
    let entity = event.event_target();

    let Ok((.., modifiers)) = capacity.get(entity) else {
        debug!("Rejecting remove capacity modifier intent for {entity:?}: missing Capacity");

        commands.trigger(RemoveCapacityModifierRejected {
            entity,
            error: RemoveCapacityModifierIntentError::MissingCapacity,
        });
        return;
    };

    let Some(modifiers) = modifiers else {
        debug!(
            "Rejecting remove capacity modifier intent for {entity:?}: missing CapacityModifiers"
        );

        commands.trigger(RemoveCapacityModifierRejected {
            entity,
            error: RemoveCapacityModifierIntentError::MissingCapacityModifiers,
        });
        return;
    };

    let Some(modifiers) = modifiers.without(event.id) else {
        debug!("Rejecting remove capacity modifier intent for {entity:?}: missing modifier");

        commands.trigger(RemoveCapacityModifierRejected {
            entity,
            error: RemoveCapacityModifierIntentError::MissingModifier,
        });
        return;
    };

    commands.entity(entity).insert(modifiers);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capacity::{
        add_capacity_modifier::{AddCapacityModifierIntent, AddCapacityModifierPlugin},
        sync_capacity::SyncCapacityPlugin,
    };

    #[derive(Resource)]
    struct LastRemoveCapacityModifierRejection {
        error: RemoveCapacityModifierIntentError,
    }

    fn record_remove_capacity_modifier_rejection(
        event: On<RemoveCapacityModifierRejected>,
        mut commands: Commands,
    ) {
        commands.insert_resource(LastRemoveCapacityModifierRejection { error: event.error });
    }

    #[test]
    fn should_remove_capacity_modifier() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AddCapacityModifierPlugin,
            RemoveCapacityModifierPlugin,
            SyncCapacityPlugin,
        ));

        let modifier = CapacityModifier::new(50);
        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.world_mut()
            .trigger(AddCapacityModifierIntent { entity, modifier });

        app.update();

        app.world_mut().trigger(RemoveCapacityModifierIntent {
            entity,
            id: modifier.id(),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<MaxCapacity>(entity)
                .expect("MaxCapacity should exist"),
            400
        );
    }

    #[test]
    fn should_reject_remove_capacity_modifier_when_modifier_is_missing() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RemoveCapacityModifierPlugin));
        app.add_observer(record_remove_capacity_modifier_rejection);

        let entity = app.world_mut().spawn(Capacity(400)).id();

        app.world_mut().trigger(RemoveCapacityModifierIntent {
            entity,
            id: CapacityModifierId(suon_uuid::UuidGenerator::generate_uuid()),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastRemoveCapacityModifierRejection>()
                .error,
            RemoveCapacityModifierIntentError::MissingModifier
        );
    }

    #[test]
    fn should_reject_remove_capacity_modifier_without_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RemoveCapacityModifierPlugin));
        app.add_observer(record_remove_capacity_modifier_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(RemoveCapacityModifierIntent {
            entity,
            id: CapacityModifierId(suon_uuid::UuidGenerator::generate_uuid()),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastRemoveCapacityModifierRejection>()
                .error,
            RemoveCapacityModifierIntentError::MissingCapacity
        );
    }
}
