use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a damage intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageIntentError {
    /// The target entity does not have both [`Health`] and [`MaxHealth`].
    MissingHealthComponents,

    /// The requested amount is zero and would not change health.
    EmptyAmount,

    /// Health is already zero.
    AlreadyDead,
}

/// Intent requesting health reduction for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct DamageIntent {
    /// Entity whose health should be reduced.
    #[event_target]
    pub entity: Entity,

    /// Health amount to subtract.
    pub amount: u32,
}

/// Event emitted after health is reduced.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Damage {
    /// Entity whose health was reduced.
    #[event_target]
    entity: Entity,

    /// Health before applying the intent.
    pub previous: Health,
}

/// Event emitted when a damage intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct DamageRejected {
    /// Entity whose damage intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: DamageIntentError,
}

pub(crate) struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_damage_intent);
    }
}

fn apply_damage_intent(
    event: On<DamageIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Health, &MaxHealth)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting damage_intent for {entity:?}: empty amount");

        commands.trigger(DamageRejected {
            entity,
            amount: event.amount,
            error: DamageIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, ..)) = vitals.get_mut(entity) else {
        debug!("Rejecting damage_intent for {entity:?}: missing components");

        commands.trigger(DamageRejected {
            entity,
            amount: event.amount,
            error: DamageIntentError::MissingHealthComponents,
        });
        return;
    };

    if current.is_zero() {
        debug!("Rejecting damage_intent for {entity:?}: already zero");

        commands.trigger(DamageRejected {
            entity,
            amount: event.amount,
            error: DamageIntentError::AlreadyDead,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_sub(event.amount);
    **current = next;

    trace!(
        "Applied damage for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(Damage { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Health,
    }

    fn record(event: On<Damage>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: DamageIntentError,
    }

    fn record_rejection(event: On<DamageRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(75), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            50
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Health(75));
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(DamageIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            DamageIntentError::MissingHealthComponents
        );
    }

    #[test]
    fn should_reject_empty_amount() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Health(50), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            DamageIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_when_already_dead() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Health(0), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            DamageIntentError::AlreadyDead
        );
    }

    #[test]
    fn should_saturate_at_zero_when_damage_exceeds_health() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, DamagePlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Health(10), MaxHealth(100))).id();

        app.world_mut().trigger(DamageIntent {
            entity,
            amount: 999,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Health>(entity)
                .expect("Health should exist"),
            0
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Health(10));
    }
}
