use bevy::prelude::*;
use log::debug;

use super::*;

/// Reason why an add speed modifier intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddSpeedModifierIntentError {
    /// The target entity does not have [`BaseSpeed`].
    MissingBaseSpeed,

    /// The target entity does not have [`SpeedModifiers`].
    MissingSpeedModifiers,
}

/// Intent requesting a speed modifier to be added or replaced for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AddSpeedModifierIntent {
    /// Entity whose speed modifier should be added or replaced.
    #[event_target]
    pub entity: Entity,

    /// Modifier to add or replace.
    pub modifier: SpeedModifier,
}

/// Event emitted when an add speed modifier intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AddSpeedModifierRejected {
    /// Entity whose add speed modifier intent was rejected.
    #[event_target]
    entity: Entity,

    /// Rejection reason.
    pub error: AddSpeedModifierIntentError,
}

pub(crate) struct AddSpeedModifierPlugin;

impl Plugin for AddSpeedModifierPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_add_speed_modifier_intent);
    }
}

fn apply_add_speed_modifier_intent(
    event: On<AddSpeedModifierIntent>,
    mut commands: Commands,
    speeds: Query<(&BaseSpeed, Option<&SpeedModifiers>)>,
) {
    let entity = event.event_target();

    let Ok((.., modifiers)) = speeds.get(entity) else {
        debug!("Rejecting add speed modifier intent for {entity:?}: missing BaseSpeed");

        commands.trigger(AddSpeedModifierRejected {
            entity,
            error: AddSpeedModifierIntentError::MissingBaseSpeed,
        });
        return;
    };

    let Some(modifiers) = modifiers else {
        debug!("Rejecting add speed modifier intent for {entity:?}: missing SpeedModifiers");

        commands.trigger(AddSpeedModifierRejected {
            entity,
            error: AddSpeedModifierIntentError::MissingSpeedModifiers,
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
    use crate::speed::sync_speed::SyncSpeedPlugin;

    #[test]
    fn should_add_speed_modifier() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddSpeedModifierPlugin, SyncSpeedPlugin));

        let entity = app.world_mut().spawn((BaseSpeed(220), Speed(220))).id();

        app.world_mut().trigger(AddSpeedModifierIntent {
            entity,
            modifier: SpeedModifier::new(-250),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            -30
        );

        assert_eq!(
            app.world()
                .get::<SpeedModifiers>(entity)
                .expect("SpeedModifiers should exist")
                .iter()
                .count(),
            1
        );
    }

    #[derive(Resource)]
    struct LastAddSpeedModifierRejection {
        error: AddSpeedModifierIntentError,
    }

    fn record_rejection(event: On<AddSpeedModifierRejected>, mut commands: Commands) {
        commands.insert_resource(LastAddSpeedModifierRejection { error: event.error });
    }

    #[test]
    fn should_reject_add_speed_modifier_without_base_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddSpeedModifierPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(AddSpeedModifierIntent {
            entity,
            modifier: SpeedModifier::new(50),
        });

        app.update();

        assert_eq!(
            app.world()
                .resource::<LastAddSpeedModifierRejection>()
                .error,
            AddSpeedModifierIntentError::MissingBaseSpeed
        );
    }

    #[test]
    fn should_replace_speed_modifier_with_same_id() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddSpeedModifierPlugin, SyncSpeedPlugin));

        let modifier = SpeedModifier::new(50);

        let entity = app
            .world_mut()
            .spawn((BaseSpeed(220), SpeedModifiers::new([modifier]), Speed(270)))
            .id();

        let replacement = SpeedModifier {
            id: modifier.id,
            value: 100,
        };

        app.world_mut().trigger(AddSpeedModifierIntent {
            entity,
            modifier: replacement,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            320
        );

        assert_eq!(
            app.world()
                .get::<SpeedModifiers>(entity)
                .expect("SpeedModifiers should exist")
                .iter()
                .count(),
            1
        );
    }
}
