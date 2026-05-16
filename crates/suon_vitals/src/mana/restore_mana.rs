use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a restore mana intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreManaIntentError {
    /// The target entity does not have both [`Mana`] and [`MaxMana`].
    MissingManaComponents,

    /// The requested amount is zero and would not change mana.
    EmptyAmount,

    /// Mana is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting mana restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreManaIntent {
    /// Entity whose mana should be restored.
    #[event_target]
    pub entity: Entity,

    /// Mana amount to restore.
    pub amount: u32,
}

/// Event emitted after mana is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreMana {
    /// Entity whose mana was restored.
    #[event_target]
    entity: Entity,

    /// Mana before applying the intent.
    pub previous: Mana,
}

/// Event emitted when a restore mana intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreManaRejected {
    /// Entity whose restore mana intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: RestoreManaIntentError,
}

pub(crate) struct RestoreManaPlugin;

impl Plugin for RestoreManaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_restore_mana_intent);
    }
}

fn apply_restore_mana_intent(
    event: On<RestoreManaIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Mana, &MaxMana)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting restore_mana_intent for {entity:?}: empty amount");

        commands.trigger(RestoreManaRejected {
            entity,
            amount: event.amount,
            error: RestoreManaIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug!("Rejecting restore_mana_intent for {entity:?}: missing components");

        commands.trigger(RestoreManaRejected {
            entity,
            amount: event.amount,
            error: RestoreManaIntentError::MissingManaComponents,
        });
        return;
    };

    if current.is_at_maximum(maximum) {
        debug!("Rejecting restore_mana_intent for {entity:?}: already at maximum");

        commands.trigger(RestoreManaRejected {
            entity,
            amount: event.amount,
            error: RestoreManaIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_add(event.amount).min(**maximum);
    **current = next;

    trace!(
        "Applied restore_mana for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(RestoreMana { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Mana,
    }

    fn record(event: On<RestoreMana>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: RestoreManaIntentError,
    }

    fn record_rejection(event: On<RestoreManaRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(50), MaxMana(100))).id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            75
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Mana(50));
    }

    #[test]
    fn should_reject_empty_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaIntentError::MissingManaComponents
        );
    }

    #[test]
    fn should_reject_when_already_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Mana(100), MaxMana(100))).id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_restore_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Mana(90), MaxMana(100))).id();

        app.world_mut()
            .trigger(RestoreManaIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            **app.world().get::<Mana>(entity).expect("Mana should exist"),
            100
        );
    }
}
