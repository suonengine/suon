use bevy::prelude::*;
use log::debug;

use super::*;

/// Reason why a set base speed intent was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetBaseSpeedIntentError {
    /// The target entity does not have [`BaseSpeed`].
    MissingBaseSpeed,
}

/// Intent requesting a base speed change for the target entity.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SetBaseSpeedIntent {
    /// Entity whose base speed should be changed.
    #[event_target]
    pub entity: Entity,

    /// New base speed value.
    pub value: BaseSpeed,
}

/// Event emitted when a set base speed intent cannot be applied.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SetBaseSpeedRejected {
    /// Entity whose set base speed intent was rejected.
    #[event_target]
    entity: Entity,

    /// Rejection reason.
    pub error: SetBaseSpeedIntentError,
}

pub(crate) struct SetBaseSpeedPlugin;

impl Plugin for SetBaseSpeedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(apply_set_base_speed_intent);
    }
}

fn apply_set_base_speed_intent(
    event: On<SetBaseSpeedIntent>,
    mut commands: Commands,
    speeds: Query<&BaseSpeed>,
) {
    let entity = event.event_target();

    if speeds.get(entity).is_err() {
        debug!("Rejecting set base speed intent for {entity:?}: missing BaseSpeed");

        commands.trigger(SetBaseSpeedRejected {
            entity,
            error: SetBaseSpeedIntentError::MissingBaseSpeed,
        });
        return;
    }

    commands.entity(entity).insert(event.value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speed::sync_speed::SyncSpeedPlugin;

    #[test]
    fn should_set_base_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SetBaseSpeedPlugin, SyncSpeedPlugin));

        let entity = app
            .world_mut()
            .spawn((
                BaseSpeed(220),
                SpeedModifiers::new([SpeedModifier::new(10)]),
                Speed(230),
            ))
            .id();

        app.world_mut().trigger(SetBaseSpeedIntent {
            entity,
            value: BaseSpeed(300),
        });

        app.update();

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            310
        );
    }

    #[derive(Resource)]
    struct LastSetBaseSpeedRejection {
        error: SetBaseSpeedIntentError,
    }

    fn record_rejection(event: On<SetBaseSpeedRejected>, mut commands: Commands) {
        commands.insert_resource(LastSetBaseSpeedRejection { error: event.error });
    }

    #[test]
    fn should_reject_set_base_speed_without_base_speed() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SetBaseSpeedPlugin));
        app.add_observer(record_rejection);

        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().trigger(SetBaseSpeedIntent {
            entity,
            value: BaseSpeed(100),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastSetBaseSpeedRejection>().error,
            SetBaseSpeedIntentError::MissingBaseSpeed
        );
    }
}
