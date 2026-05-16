use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a restore capacity intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreCapacityIntentError {
    /// The target entity does not have both [`MaxCapacity`] and [`FreeCapacity`].
    MissingCapacityComponents,

    /// The requested amount is zero and would not change capacity.
    EmptyAmount,

    /// Free capacity is already at maximum.
    AlreadyAtMaximum,
}

/// Intent requesting free capacity restoration for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreCapacityIntent {
    /// Entity whose free capacity should be restored.
    #[event_target]
    pub entity: Entity,

    /// Free capacity amount to restore.
    pub amount: u32,
}

/// Event emitted after free capacity is restored.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreCapacity {
    /// Entity whose free capacity was restored.
    #[event_target]
    entity: Entity,

    /// Free capacity before applying the intent.
    pub previous: FreeCapacity,
}

/// Event emitted when a restore capacity intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreCapacityRejected {
    /// Entity whose restore capacity intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: RestoreCapacityIntentError,
}

pub(crate) struct RestoreCapacityPlugin;

impl Plugin for RestoreCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_restore_capacity_intent);
    }
}

fn apply_restore_capacity_intent(
    event: On<RestoreCapacityIntent>,
    mut commands: Commands,
    mut capacity: Query<(&MaxCapacity, &mut FreeCapacity)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting restore capacity intent for {entity:?}: empty amount");

        commands.trigger(RestoreCapacityRejected {
            entity,
            amount: event.amount,
            error: RestoreCapacityIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((maximum, mut free_capacity)) = capacity.get_mut(entity) else {
        debug!(
            "Rejecting restore capacity intent for {entity:?}: missing MaxCapacity or FreeCapacity"
        );

        commands.trigger(RestoreCapacityRejected {
            entity,
            amount: event.amount,
            error: RestoreCapacityIntentError::MissingCapacityComponents,
        });
        return;
    };

    if free_capacity.is_at_maximum(maximum) {
        debug!(
            "Rejecting restore capacity intent for {entity:?}: free capacity is already at maximum"
        );

        commands.trigger(RestoreCapacityRejected {
            entity,
            amount: event.amount,
            error: RestoreCapacityIntentError::AlreadyAtMaximum,
        });
        return;
    }

    let previous = *free_capacity;
    let next = free_capacity.saturating_add(event.amount).min(**maximum);
    **free_capacity = next;

    trace!(
        "Applied capacity restore for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(RestoreCapacity { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastRestoreCapacity {
        previous: FreeCapacity,
    }

    fn record_restore_capacity(event: On<RestoreCapacity>, mut commands: Commands) {
        commands.insert_resource(LastRestoreCapacity {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastRestoreCapacityRejection {
        error: RestoreCapacityIntentError,
    }

    fn record_restore_capacity_rejection(
        event: On<RestoreCapacityRejected>,
        mut commands: Commands,
    ) {
        commands.insert_resource(LastRestoreCapacityRejection { error: event.error });
    }

    #[test]
    fn should_restore_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreCapacityPlugin));
        app.add_observer(record_restore_capacity);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(300)))
            .id();

        app.world_mut()
            .trigger(RestoreCapacityIntent { entity, amount: 50 });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            350
        );

        assert_eq!(*app.world().resource::<LastRestoreCapacity>().previous, 300);
    }

    #[test]
    fn should_reject_empty_restore_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreCapacityPlugin));
        app.add_observer(record_restore_capacity_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreCapacityIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRestoreCapacityRejection>().error,
            RestoreCapacityIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_reject_restore_capacity_when_at_maximum() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreCapacityPlugin));
        app.add_observer(record_restore_capacity_rejection);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(400)))
            .id();

        app.world_mut()
            .trigger(RestoreCapacityIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRestoreCapacityRejection>().error,
            RestoreCapacityIntentError::AlreadyAtMaximum
        );
    }

    #[test]
    fn should_reject_restore_capacity_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreCapacityPlugin));
        app.add_observer(record_restore_capacity_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(RestoreCapacityIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastRestoreCapacityRejection>().error,
            RestoreCapacityIntentError::MissingCapacityComponents
        );
    }

    #[test]
    fn should_clamp_to_maximum_when_restore_exceeds_remaining() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreCapacityPlugin));
        app.add_observer(record_restore_capacity);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(380)))
            .id();

        app.world_mut().trigger(RestoreCapacityIntent {
            entity,
            amount: 100,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            400
        );
    }
}
