use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a consume mana intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumeManaIntentError {
    /// The target entity does not have both [`Mana`] and [`MaxMana`].
    MissingManaComponents,

    /// The requested amount is zero and would not change mana.
    EmptyAmount,

    /// Mana is already zero.
    AlreadyEmpty,
}

/// Intent requesting mana consumption for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeManaIntent {
    /// Entity whose mana should be consumed.
    #[event_target]
    pub entity: Entity,

    /// Mana amount to consume.
    pub amount: u32,
}

/// Event emitted after mana is consumed.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeMana {
    /// Entity whose mana was consumed.
    #[event_target]
    entity: Entity,

    /// Mana before applying the intent.
    pub previous: Mana,
}

/// Event emitted when a consume mana intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeManaRejected {
    /// Entity whose consume mana intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: ConsumeManaIntentError,
}

pub(crate) struct ConsumeManaPlugin;

impl Plugin for ConsumeManaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_consume_mana_intent);
    }
}

fn apply_consume_mana_intent(
    event: On<ConsumeManaIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Mana, &MaxMana)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting consume_mana_intent for {entity:?}: empty amount");

        commands.trigger(ConsumeManaRejected {
            entity,
            amount: event.amount,
            error: ConsumeManaIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, ..)) = vitals.get_mut(entity) else {
        debug!("Rejecting consume_mana_intent for {entity:?}: missing components");

        commands.trigger(ConsumeManaRejected {
            entity,
            amount: event.amount,
            error: ConsumeManaIntentError::MissingManaComponents,
        });
        return;
    };

    if current.is_zero() {
        debug!("Rejecting consume_mana_intent for {entity:?}: already zero");

        commands.trigger(ConsumeManaRejected {
            entity,
            amount: event.amount,
            error: ConsumeManaIntentError::AlreadyEmpty,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_sub(event.amount);
    **current = next;

    trace!(
        "Applied consume_mana for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(ConsumeMana { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Mana,
    }

    fn record(event: On<ConsumeMana>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: ConsumeManaIntentError,
    }

    fn record_rejection(event: On<ConsumeManaRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(75), MaxMana(100))).id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            50
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Mana(75));
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeManaIntentError::MissingManaComponents
        );
    }

    #[test]
    fn should_reject_empty_amount() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Mana(50), MaxMana(100))).id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeManaIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_when_already_empty() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Mana(0), MaxMana(100))).id();

        app.world_mut()
            .trigger(ConsumeManaIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeManaIntentError::AlreadyEmpty
        );
    }

    #[test]
    fn should_saturate_at_zero_when_consume_exceeds_mana() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeManaPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(10), MaxMana(100))).id();

        app.world_mut().trigger(ConsumeManaIntent {
            entity,
            amount: 999,
        });

        app.update();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            0
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Mana(10));
    }
}
