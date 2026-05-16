use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a consume soul intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumeSoulIntentError {
    /// The target entity does not have both [`Soul`] and [`MaxSoul`].
    MissingSoulComponents,
    /// The requested amount is zero and would not change soul.
    EmptyAmount,
    /// Soul is already zero.
    AlreadyEmpty,
}

/// Intent requesting soul consumption for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeSoulIntent {
    /// Entity whose soul should be consumed.
    #[event_target]
    pub entity: Entity,
    /// Soul amount to consume.
    pub amount: u32,
}

/// Event emitted after soul is consumed.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeSoul {
    /// Entity whose soul was consumed.
    #[event_target]
    entity: Entity,
    /// Soul before applying the intent.
    pub previous: Soul,
}

/// Event emitted when a consume soul intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeSoulRejected {
    /// Entity whose consume soul intent was rejected.
    #[event_target]
    entity: Entity,
    /// Requested amount.
    pub amount: u32,
    /// Rejection reason.
    pub error: ConsumeSoulIntentError,
}

pub(crate) struct ConsumeSoulPlugin;

impl Plugin for ConsumeSoulPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_consume_soul_intent);
    }
}

fn apply_consume_soul_intent(
    event: On<ConsumeSoulIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Soul, &MaxSoul)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting consume_soul_intent for {entity:?}: empty amount");

        commands.trigger(ConsumeSoulRejected {
            entity,
            amount: event.amount,
            error: ConsumeSoulIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, ..)) = vitals.get_mut(entity) else {
        debug!("Rejecting consume_soul_intent for {entity:?}: missing components");

        commands.trigger(ConsumeSoulRejected {
            entity,
            amount: event.amount,
            error: ConsumeSoulIntentError::MissingSoulComponents,
        });
        return;
    };

    if current.is_zero() {
        debug!("Rejecting consume_soul_intent for {entity:?}: already zero");

        commands.trigger(ConsumeSoulRejected {
            entity,
            amount: event.amount,
            error: ConsumeSoulIntentError::AlreadyEmpty,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_sub(event.amount);
    **current = next;

    trace!(
        "Applied consume_soul for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(ConsumeSoul { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Soul,
    }

    fn record(event: On<ConsumeSoul>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: ConsumeSoulIntentError,
    }

    fn record_rejection(event: On<ConsumeSoulRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_consume() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(75), MaxSoul(100))).id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            50
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Soul(75));
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeSoulIntentError::MissingSoulComponents
        );
    }

    #[test]
    fn should_reject_empty_amount() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Soul(50), MaxSoul(100))).id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeSoulIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_when_already_empty() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Soul(0), MaxSoul(100))).id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            ConsumeSoulIntentError::AlreadyEmpty
        );
    }

    #[test]
    fn should_saturate_at_zero_when_consume_exceeds_soul() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(10), MaxSoul(100))).id();

        app.world_mut().trigger(ConsumeSoulIntent {
            entity,
            amount: 999,
        });

        app.update();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            0
        );
    }
}
