use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a restore soul intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreSoulIntentError {
    /// The target entity does not have both [`Soul`] and [`MaxSoul`].
    MissingSoulComponents,

    /// The requested amount is zero and would not change soul.
    EmptyAmount,

    /// Soul is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting soul restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreSoulIntent {
    /// Entity whose soul should be restored.
    #[event_target]
    pub entity: Entity,

    /// Soul amount to restore.
    pub amount: u32,
}

/// Event emitted after soul is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreSoul {
    /// Entity whose soul was restored.
    #[event_target]
    entity: Entity,

    /// Soul before applying the intent.
    pub previous: Soul,
}

/// Event emitted when a restore soul intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreSoulRejected {
    /// Entity whose restore soul intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: RestoreSoulIntentError,
}

pub(crate) struct RestoreSoulPlugin;

impl Plugin for RestoreSoulPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_restore_soul_intent);
    }
}

fn apply_restore_soul_intent(
    event: On<RestoreSoulIntent>,
    mut commands: Commands,
    mut vitals: Query<(&mut Soul, &MaxSoul)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting restore_soul_intent for {entity:?}: empty amount");

        commands.trigger(RestoreSoulRejected {
            entity,
            amount: event.amount,
            error: RestoreSoulIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((mut current, maximum)) = vitals.get_mut(entity) else {
        debug!("Rejecting restore_soul_intent for {entity:?}: missing components");

        commands.trigger(RestoreSoulRejected {
            entity,
            amount: event.amount,
            error: RestoreSoulIntentError::MissingSoulComponents,
        });
        return;
    };

    if current.is_at_maximum(maximum) {
        debug!("Rejecting restore_soul_intent for {entity:?}: already at maximum");

        commands.trigger(RestoreSoulRejected {
            entity,
            amount: event.amount,
            error: RestoreSoulIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *current;
    let next = current.saturating_add(event.amount).min(**maximum);
    **current = next;

    trace!(
        "Applied restore_soul for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(RestoreSoul { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastResult {
        previous: Soul,
    }

    fn record(event: On<RestoreSoul>, mut commands: Commands) {
        commands.insert_resource(LastResult {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRejection {
        error: RestoreSoulIntentError,
    }

    fn record_rejection(event: On<RestoreSoulRejected>, mut commands: Commands) {
        commands.insert_resource(LastRejection { error: event.error });
    }

    #[test]
    fn should_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(50), MaxSoul(100))).id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 25 });

        app.update();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            75
        );

        assert_eq!(app.world().resource::<LastResult>().previous, Soul(50));
    }

    #[test]
    fn should_reject_empty_restore() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreSoulIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreSoulIntentError::MissingSoulComponents
        );
    }

    #[test]
    fn should_reject_when_already_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn((Soul(100), MaxSoul(100))).id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRejection>().error,
            RestoreSoulIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_restore_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(90), MaxSoul(100))).id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            **app.world().get::<Soul>(entity).expect("Soul should exist"),
            100
        );
    }
}
