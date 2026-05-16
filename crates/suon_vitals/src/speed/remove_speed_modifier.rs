use bevy::prelude::*;
use log::debug;

use super::*;

/// Reason why a remove speed modifier intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveSpeedModifierIntentError {
    /// The target entity does not have [`BaseSpeed`].
    MissingBaseSpeed,
    /// The target entity does not have [`SpeedModifiers`].
    MissingSpeedModifiers,
    /// The target entity does not have a modifier with the requested id.
    MissingModifier,
}

/// Intent requesting a speed modifier removal for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RemoveSpeedModifierIntent {
    /// Entity whose speed modifier should be removed.
    #[event_target]
    pub entity: Entity,
    /// Modifier id to remove.
    pub id: SpeedModifierId,
}

/// Event emitted when a remove speed modifier intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RemoveSpeedModifierRejected {
    /// Entity whose remove speed modifier intent was rejected.
    #[event_target]
    entity: Entity,
    /// Rejection reason.
    pub error: RemoveSpeedModifierIntentError,
}

pub(crate) struct RemoveSpeedModifierPlugin;

impl Plugin for RemoveSpeedModifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_remove_speed_modifier_intent);
    }
}

fn apply_remove_speed_modifier_intent(
    event: On<RemoveSpeedModifierIntent>,
    mut commands: Commands,
    speeds: Query<(&BaseSpeed, Option<&SpeedModifiers>)>,
) {
    let entity = event.event_target();

    let Ok((.., modifiers)) = speeds.get(entity) else {
        debug!("Rejecting remove speed modifier intent for {entity:?}: missing BaseSpeed");
        commands.trigger(RemoveSpeedModifierRejected {
            entity,
            error: RemoveSpeedModifierIntentError::MissingBaseSpeed,
        });
        return;
    };

    let Some(modifiers) = modifiers else {
        debug!("Rejecting remove speed modifier intent for {entity:?}: missing SpeedModifiers");
        commands.trigger(RemoveSpeedModifierRejected {
            entity,
            error: RemoveSpeedModifierIntentError::MissingSpeedModifiers,
        });
        return;
    };

    let Some(modifiers) = modifiers.without(event.id) else {
        debug!("Rejecting remove speed modifier intent for {entity:?}: missing modifier");
        commands.trigger(RemoveSpeedModifierRejected {
            entity,
            error: RemoveSpeedModifierIntentError::MissingModifier,
        });
        return;
    };

    commands.entity(entity).insert(modifiers);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speed::sync_speed::SyncSpeedPlugin;

    #[test]
    fn should_remove_speed_modifier() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RemoveSpeedModifierPlugin, SyncSpeedPlugin));

        let modifier = SpeedModifier::new(-50);

        let entity = app
            .world_mut()
            .spawn((BaseSpeed(220), SpeedModifiers::new([modifier]), Speed(170)))
            .id();

        app.world_mut().trigger(RemoveSpeedModifierIntent {
            entity,
            id: modifier.id(),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            220
        );

        assert!(
            app.world()
                .get::<SpeedModifiers>(entity)
                .expect("SpeedModifiers should exist")
                .is_empty()
        );
    }

    #[derive(Resource)]
    struct LastRemoveSpeedModifierRejection {
        error: RemoveSpeedModifierIntentError,
    }

    fn record_rejection(event: On<RemoveSpeedModifierRejected>, mut commands: Commands) {
        commands.insert_resource(LastRemoveSpeedModifierRejection { error: event.error });
    }

    #[test]
    fn should_reject_remove_speed_modifier_without_base_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RemoveSpeedModifierPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(RemoveSpeedModifierIntent {
            entity,
            id: SpeedModifierId(suon_uuid::UuidGenerator::generate_uuid()),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastRemoveSpeedModifierRejection>()
                .error,
            RemoveSpeedModifierIntentError::MissingBaseSpeed
        );
    }

    #[test]
    fn should_reject_remove_speed_modifier_when_modifier_not_found() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RemoveSpeedModifierPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((BaseSpeed(220), Speed(220))).id();

        app.world_mut().trigger(RemoveSpeedModifierIntent {
            entity,
            id: SpeedModifierId(suon_uuid::UuidGenerator::generate_uuid()),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastRemoveSpeedModifierRejection>()
                .error,
            RemoveSpeedModifierIntentError::MissingModifier
        );
    }
}
