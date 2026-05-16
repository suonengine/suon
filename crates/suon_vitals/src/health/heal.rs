use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a heal intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealIntentError {
    /// The target entity does not have both [`Health`] and [`MaxHealth`].
    MissingHealthComponents,
    /// The requested amount is zero and would not change health.
    EmptyAmount,
    /// Health is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting health restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HealIntent {
    /// Entity whose health should be restored.
    #[event_target]
    pub entity: Entity,

    /// Health amount to restore.
    pub amount: u32,
}

/// Event emitted after health is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Heal {
    /// Entity whose health was restored.
    #[event_target]
    entity: Entity,

    /// Health before applying the intent.
    pub previous: Health,
}

/// Event emitted when a heal intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct HealRejected {
    /// Entity whose heal intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: HealIntentError,
}

pub(crate) struct HealPlugin;

impl Plugin for HealPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_heal_intent);
    }
}

fn apply_heal_intent(
    event: On<HealIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Health, &MaxHealth)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting heal_intent for {entity:?}: empty amount");

        commands.trigger(HealRejected {
            entity,
            amount: event.amount,
            error: HealIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug!("Rejecting heal_intent for {entity:?}: missing components");

        commands.trigger(HealRejected {
            entity,
            amount: event.amount,
            error: HealIntentError::MissingHealthComponents,
        });
        return;
    };

    if current.is_at_maximum(maximum) {
        debug!("Rejecting heal_intent for {entity:?}: already at maximum");

        commands.trigger(HealRejected {
            entity,
            amount: event.amount,
            error: HealIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_add(event.amount).min(**maximum);
    **current = next;

    trace!(
        "Applied heal for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(Heal { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Health,
    }

    fn record(event: On<Heal>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: HealIntentError,
    }

    fn record_rejection(event: On<HealRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(50), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            75
        );
        assert_eq!(app.world().resource::<LastResult>().previous, Health(50));
    }

    #[test]
    fn should_reject_empty_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(HealIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            HealIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(HealIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            HealIntentError::MissingHealthComponents
        );
    }

    #[test]
    fn should_reject_when_already_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Health(100), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            HealIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_heal_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, HealPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(90), MaxHealth(100))).id();

        app.world_mut().trigger(HealIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            100
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Health(90));
    }
}
