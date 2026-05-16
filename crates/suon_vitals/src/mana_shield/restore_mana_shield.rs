use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a restore mana shield intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreManaShieldIntentError {
    /// The target entity does not have both [`ManaShield`] and [`MaxManaShield`].
    MissingManaShieldComponents,

    /// The requested amount is zero and would not change the shield.
    EmptyAmount,

    /// Mana shield is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting mana shield restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreManaShieldIntent {
    /// Entity whose mana shield should be restored.
    #[event_target]
    pub entity: Entity,

    /// Shield amount to restore.
    pub amount: u32,
}

/// Event emitted after mana shield is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreManaShield {
    /// Entity whose mana shield was restored.
    #[event_target]
    entity: Entity,

    /// Mana shield before applying the intent.
    pub previous: ManaShield,
}

/// Event emitted when a restore mana shield intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreManaShieldRejected {
    /// Entity whose restore mana shield intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: RestoreManaShieldIntentError,
}

pub(crate) struct RestoreManaShieldPlugin;

impl Plugin for RestoreManaShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_restore_mana_shield_intent);
    }
}

fn apply_restore_mana_shield_intent(
    event: On<RestoreManaShieldIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut ManaShield, &MaxManaShield)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting restore_mana_shield_intent for {entity:?}: empty amount");

        commands.trigger(RestoreManaShieldRejected {
            entity,
            amount: event.amount,
            error: RestoreManaShieldIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug!("Rejecting restore_mana_shield_intent for {entity:?}: missing components");

        commands.trigger(RestoreManaShieldRejected {
            entity,
            amount: event.amount,
            error: RestoreManaShieldIntentError::MissingManaShieldComponents,
        });
        return;
    };

    if current.is_at_maximum(maximum) {
        debug!("Rejecting restore_mana_shield_intent for {entity:?}: already at maximum");

        commands.trigger(RestoreManaShieldRejected {
            entity,
            amount: event.amount,
            error: RestoreManaShieldIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_add(event.amount).min(**maximum);
    **current = next;

    trace!(
        "Applied restore_mana_shield for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(RestoreManaShield { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: ManaShield,
    }

    fn record(event: On<RestoreManaShield>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: RestoreManaShieldIntentError,
    }

    fn record_rejection(event: On<RestoreManaShieldRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaShieldPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(50), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            75
        );

        assert_eq!(
            app.world().resource::<LastResult>().previous,
            ManaShield(50)
        );
    }

    #[test]
    fn should_reject_empty_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaShieldIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaShieldIntentError::MissingManaShieldComponents
        );
    }

    #[test]
    fn should_reject_when_already_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((ManaShield(100), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreManaShieldIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_restore_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreManaShieldPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(90), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(RestoreManaShieldIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            100
        );
    }
}
