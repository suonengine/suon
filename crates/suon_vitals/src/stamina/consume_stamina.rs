use bevy::prelude::*;
use log::{debug, trace};
use std::time::Duration;

use super::*;

/// Reason why a consume stamina intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumeStaminaIntentError {
    /// The target entity does not have both [`Stamina`] and [`MaxStamina`].
    MissingStaminaComponents,

    /// The requested amount is zero and would not change stamina.
    EmptyAmount,

    /// Stamina is already zero.
    AlreadyExhausted,
}

/// Intent requesting stamina consumption for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeStaminaIntent {
    /// Entity whose stamina should be consumed.
    #[event_target]
    pub entity: Entity,

    /// Stamina duration to consume.
    pub amount: Duration,
}

/// Event emitted after stamina is consumed.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeStamina {
    /// Entity whose stamina was consumed.
    #[event_target]
    entity: Entity,

    /// Stamina before applying the intent.
    pub previous: Stamina,
}

/// Event emitted when a consume stamina intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeStaminaRejected {
    /// Entity whose consume stamina intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested duration.
    pub amount: Duration,

    /// Rejection reason.
    pub error: ConsumeStaminaIntentError,
}

pub(crate) struct ConsumeStaminaPlugin;

impl Plugin for ConsumeStaminaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_consume_stamina_intent);
    }
}

fn apply_consume_stamina_intent(
    event: On<ConsumeStaminaIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Stamina, &MaxStamina)>,
) {
    let entity = event.event_target();

    if event.amount == Duration::ZERO {
        debug!("Rejecting consume_stamina_intent for {entity:?}: empty amount");

        commands.trigger(ConsumeStaminaRejected {
            entity,
            amount: event.amount,
            error: ConsumeStaminaIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, ..)) = vitals.get_mut(entity) else {
        debug!("Rejecting consume_stamina_intent for {entity:?}: missing components");

        commands.trigger(ConsumeStaminaRejected {
            entity,
            amount: event.amount,
            error: ConsumeStaminaIntentError::MissingStaminaComponents,
        });
        return;
    };

    if current.is_zero() {
        debug!("Rejecting consume_stamina_intent for {entity:?}: already zero");

        commands.trigger(ConsumeStaminaRejected {
            entity,
            amount: event.amount,
            error: ConsumeStaminaIntentError::AlreadyExhausted,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_sub(event.amount);
    **current = next;

    trace!(
        "Applied consume_stamina for {entity:?}: {:?} -> {:?} (requested {:?})",
        *previous, next, event.amount
    );

    commands.trigger(ConsumeStamina { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Stamina,
    }

    fn record(event: On<ConsumeStamina>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: ConsumeStaminaIntentError,
    }

    fn record_rejection(event: On<ConsumeStaminaRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(75), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(25 * 60),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::from_secs(50 * 60)
        );

        assert_eq!(
            app.world().resource::<LastResult>().previous,
            Stamina::from_minutes(75)
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(1),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeStaminaIntentError::MissingStaminaComponents
        );
    }

    #[test]
    fn should_reject_empty_amount() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(50), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::ZERO,
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeStaminaIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_when_already_exhausted() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((Stamina(Duration::ZERO), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(10 * 60),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeStaminaIntentError::AlreadyExhausted
        );
    }

    #[test]
    fn should_saturate_at_zero_when_consume_exceeds_stamina() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeStaminaPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((Stamina::from_minutes(10), MaxStamina::from_minutes(100)))
            .id();

        app.world_mut().trigger(ConsumeStaminaIntent {
            entity,
            amount: Duration::from_secs(999 * 60),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Stamina>(entity)
                .expect("Stamina should exist"),
            Duration::ZERO
        );
    }
}
