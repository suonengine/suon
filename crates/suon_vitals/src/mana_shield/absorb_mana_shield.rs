use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why an absorb mana shield intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbsorbManaShieldIntentError {
    /// The target entity does not have both [`ManaShield`] and [`MaxManaShield`].
    MissingManaShieldComponents,

    /// The requested amount is zero and would not change the shield.
    EmptyAmount,

    /// Mana shield is already zero.
    AlreadyBroken,
}

/// Intent requesting mana shield absorption for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbsorbManaShieldIntent {
    /// Entity whose mana shield should absorb damage.
    #[event_target]
    pub entity: Entity,

    /// Shield amount to absorb.
    pub amount: u32,
}

/// Event emitted after mana shield absorbs damage.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbsorbManaShield {
    /// Entity whose mana shield absorbed damage.
    #[event_target]
    entity: Entity,

    /// Mana shield before applying the intent.
    pub previous: ManaShield,
}

/// Event emitted when an absorb mana shield intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbsorbManaShieldRejected {
    /// Entity whose absorb mana shield intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: AbsorbManaShieldIntentError,
}

pub(crate) struct AbsorbManaShieldPlugin;

impl Plugin for AbsorbManaShieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_absorb_mana_shield_intent);
    }
}

fn apply_absorb_mana_shield_intent(
    event: On<AbsorbManaShieldIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut ManaShield, &MaxManaShield)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting absorb_mana_shield_intent for {entity:?}: empty amount");

        commands.trigger(AbsorbManaShieldRejected {
            entity,
            amount: event.amount,
            error: AbsorbManaShieldIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, ..)) = vitals.get_mut(entity) else {
        debug!("Rejecting absorb_mana_shield_intent for {entity:?}: missing components");

        commands.trigger(AbsorbManaShieldRejected {
            entity,
            amount: event.amount,
            error: AbsorbManaShieldIntentError::MissingManaShieldComponents,
        });
        return;
    };

    if current.is_zero() {
        debug!("Rejecting absorb_mana_shield_intent for {entity:?}: already zero");

        commands.trigger(AbsorbManaShieldRejected {
            entity,
            amount: event.amount,
            error: AbsorbManaShieldIntentError::AlreadyBroken,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_sub(event.amount);
    **current = next;

    trace!(
        "Applied absorb_mana_shield for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(AbsorbManaShield { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: ManaShield,
    }

    fn record(event: On<AbsorbManaShield>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: AbsorbManaShieldIntentError,
    }

    fn record_rejection(event: On<AbsorbManaShieldRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AbsorbManaShieldPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(75), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            50
        );

        assert_eq!(
            app.world().resource::<LastResult>().previous,
            ManaShield(75)
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AbsorbManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            AbsorbManaShieldIntentError::MissingManaShieldComponents
        );
    }

    #[test]
    fn should_reject_empty_amount() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AbsorbManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((ManaShield(50), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            AbsorbManaShieldIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_when_already_broken() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AbsorbManaShieldPlugin));
        app.add_observer(record_rejection);

        let entity = app
            .world_mut()
            .spawn((ManaShield(0), MaxManaShield(100)))
            .id();

        app.world_mut()
            .trigger(AbsorbManaShieldIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            AbsorbManaShieldIntentError::AlreadyBroken
        );
    }

    #[test]
    fn should_saturate_at_zero_when_absorb_exceeds_shield() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AbsorbManaShieldPlugin));
        app.add_observer(record);

        let entity = app
            .world_mut()
            .spawn((ManaShield(10), MaxManaShield(100)))
            .id();

        app.world_mut().trigger(AbsorbManaShieldIntent {
            entity,
            amount: 999,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<ManaShield>(entity)
                .expect("ManaShield should exist"),
            0
        );
    }
}
