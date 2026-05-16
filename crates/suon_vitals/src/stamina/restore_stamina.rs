use bevy::prelude::*;
use log::{debug, trace};
use std::time::Duration;

use super::*;

/// Reason why a restore stamina intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreStaminaIntentError {
    /// The target entity does not have both [`Stamina`] and [`MaxStamina`].
    MissingStaminaComponents,

    /// The requested amount is zero and would not change stamina.
    EmptyAmount,

    /// Stamina is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting stamina restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreStaminaIntent {
    /// Entity whose stamina should be restored.
    #[event_target]
    pub entity: Entity,

    /// Stamina duration to restore.
    pub amount: Duration,
}

/// Event emitted after stamina is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreStamina {
    /// Entity whose stamina was restored.
    #[event_target]
    entity: Entity,

    /// Stamina before applying the intent.
    pub previous: Stamina,
}

/// Event emitted when a restore stamina intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreStaminaRejected {
    /// Entity whose restore stamina intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested duration.
    pub amount: Duration,

    /// Rejection reason.
    pub error: RestoreStaminaIntentError,
}

pub(crate) struct RestoreStaminaPlugin;

impl Plugin for RestoreStaminaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_restore_stamina_intent);
    }
}

fn apply_restore_stamina_intent(
    event: On<RestoreStaminaIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Stamina, &MaxStamina)>,
) {
    let entity = event.event_target();

    if event.amount == Duration::ZERO {
        debug!("Rejecting restore_stamina_intent for {entity:?}: empty amount");

        commands.trigger(RestoreStaminaRejected {
            entity,
            amount: event.amount,
            error: RestoreStaminaIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug!("Rejecting restore_stamina_intent for {entity:?}: missing components");

        commands.trigger(RestoreStaminaRejected {
            entity,
            amount: event.amount,
            error: RestoreStaminaIntentError::MissingStaminaComponents,
        });
        return;
    };

    if current.is_at_maximum(maximum) {
        debug!("Rejecting restore_stamina_intent for {entity:?}: already at maximum");

        commands.trigger(RestoreStaminaRejected {
            entity,
            amount: event.amount,
            error: RestoreStaminaIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_add(event.amount).min(**maximum);
    **current = next;

    trace!(
        "Applied restore_stamina for {entity:?}: {:?} -> {:?} (requested {:?})",
        *previous, next, event.amount
    );

    commands.trigger(RestoreStamina { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Stamina,
    }

    fn record(event: On<RestoreStamina>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: RestoreStaminaIntentError,
    }

    fn record_rejection(event: On<RestoreStaminaRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(50), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(25 * 60),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(75 * 60)
        );
        assert_eq!(
            app.world().resource::<LastResult>().previous,
            Stamina::from_minutes(50)
        );
    }

    #[test]
    fn should_reject_empty_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::ZERO,
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreStaminaIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(10 * 60),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreStaminaIntentError::MissingStaminaComponents
        );
    }

    #[test]
    fn should_reject_when_already_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(100), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(1),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreStaminaIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_restore_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreStaminaPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(90), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(RestoreStaminaIntent {
            entity,
            amount: Duration::from_secs(50 * 60),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(100 * 60)
        );
    }
}
