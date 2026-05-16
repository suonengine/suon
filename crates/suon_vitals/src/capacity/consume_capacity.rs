use bevy::prelude::*;
use log::{debug, trace};

use super::*;

/// Reason why a consume capacity intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumeCapacityIntentError {
    /// The target entity does not have both [`MaxCapacity`] and [`FreeCapacity`].
    MissingCapacityComponents,
    /// The requested amount is zero and would not change capacity.
    EmptyAmount,
    /// Free capacity is already zero.
    AlreadyFull,
}

/// Intent requesting free capacity consumption for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeCapacityIntent {
    /// Entity whose free capacity should be consumed.
    #[event_target]
    pub entity: Entity,

    /// Free capacity amount to consume.
    pub amount: u32,
}

/// Event emitted after free capacity is consumed.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeCapacity {
    /// Entity whose free capacity was consumed.
    #[event_target]
    entity: Entity,

    /// Free capacity before applying the intent.
    pub previous: FreeCapacity,
}

/// Event emitted when a consume capacity intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct ConsumeCapacityRejected {
    /// Entity whose consume capacity intent was rejected.
    #[event_target]
    entity: Entity,

    /// Requested amount.
    pub amount: u32,

    /// Rejection reason.
    pub error: ConsumeCapacityIntentError,
}

pub(crate) struct ConsumeCapacityPlugin;

impl Plugin for ConsumeCapacityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_consume_capacity_intent);
    }
}

fn apply_consume_capacity_intent(
    event: On<ConsumeCapacityIntent>,
    mut commands: Commands,
    mut capacity: Query<(&MaxCapacity, &mut FreeCapacity)>,
) {
    let entity = event.event_target();

    if event.amount == 0 {
        debug!("Rejecting consume capacity intent for {entity:?}: empty amount");

        commands.trigger(ConsumeCapacityRejected {
            entity,
            amount: event.amount,
            error: ConsumeCapacityIntentError::EmptyAmount,
        });
        return;
    }

    let Ok((.., mut free_capacity)) = capacity.get_mut(entity) else {
        debug!(
            "Rejecting consume capacity intent for {entity:?}: missing MaxCapacity or FreeCapacity"
        );

        commands.trigger(ConsumeCapacityRejected {
            entity,
            amount: event.amount,
            error: ConsumeCapacityIntentError::MissingCapacityComponents,
        });
        return;
    };

    if free_capacity.is_zero() {
        debug!("Rejecting consume capacity intent for {entity:?}: free capacity is already zero");

        commands.trigger(ConsumeCapacityRejected {
            entity,
            amount: event.amount,
            error: ConsumeCapacityIntentError::AlreadyFull,
        });
        return;
    }

    let previous = *free_capacity;
    let next = free_capacity.saturating_sub(event.amount);
    **free_capacity = next;

    trace!(
        "Applied capacity consume for {entity:?}: {} -> {} (requested {})",
        *previous, next, event.amount
    );

    commands.trigger(ConsumeCapacity { entity, previous });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct LastConsumeCapacity {
        previous: FreeCapacity,
    }

    fn record_consume_capacity(event: On<ConsumeCapacity>, mut commands: Commands) {
        commands.insert_resource(LastConsumeCapacity {
            previous: event.previous,
        });
    }

    #[derive(Resource)]
    struct LastConsumeCapacityRejection {
        error: ConsumeCapacityIntentError,
    }

    fn record_consume_capacity_rejection(
        event: On<ConsumeCapacityRejected>,
        mut commands: Commands,
    ) {
        commands.insert_resource(LastConsumeCapacityRejection { error: event.error });
    }

    #[test]
    fn should_consume_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin));
        app.add_observer(record_consume_capacity);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(400)))
            .id();

        app.world_mut().trigger(ConsumeCapacityIntent {
            entity,
            amount: 100,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            300
        );
        assert_eq!(*app.world().resource::<LastConsumeCapacity>().previous, 400);
    }

    #[test]
    fn should_reject_consume_capacity_without_components() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin));
        app.add_observer(record_consume_capacity_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .trigger(ConsumeCapacityIntent { entity, amount: 10 });

        app.update();

        assert_eq!(
            app.world().resource::<LastConsumeCapacityRejection>().error,
            ConsumeCapacityIntentError::MissingCapacityComponents
        );
    }

    #[test]
    fn should_reject_consume_capacity_when_already_full() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin));
        app.add_observer(record_consume_capacity_rejection);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(0)))
            .id();

        app.world_mut()
            .trigger(ConsumeCapacityIntent { entity, amount: 1 });

        app.update();

        assert_eq!(
            app.world().resource::<LastConsumeCapacityRejection>().error,
            ConsumeCapacityIntentError::AlreadyFull
        );
    }

    #[test]
    fn should_reject_consume_capacity_when_amount_is_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin));
        app.add_observer(record_consume_capacity_rejection);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(400)))
            .id();

        app.world_mut()
            .trigger(ConsumeCapacityIntent { entity, amount: 0 });

        app.update();

        assert_eq!(
            app.world().resource::<LastConsumeCapacityRejection>().error,
            ConsumeCapacityIntentError::EmptyAmount
        );
    }

    #[test]
    fn should_saturate_at_zero_when_consume_exceeds_free_capacity() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeCapacityPlugin));
        app.add_observer(record_consume_capacity);

        let entity = app
            .world_mut()
            .spawn((MaxCapacity(400), FreeCapacity(50)))
            .id();

        app.world_mut().trigger(ConsumeCapacityIntent {
            entity,
            amount: 999,
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<FreeCapacity>(entity)
                .expect("FreeCapacity should exist"),
            0
        );
    }
}
