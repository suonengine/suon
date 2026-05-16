use bevy::prelude::*;

use super::*;

/// Event emitted when effective speed changes.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpeedChanged {
    /// Entity whose speed changed.
    #[event_target]
    pub(crate) entity: Entity,

    /// Speed before the change.
    pub previous: Speed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speed::{
        add_speed_modifier::{AddSpeedModifierIntent, AddSpeedModifierPlugin},
        sync_speed::SyncSpeedPlugin,
    };

    #[derive(Resource)]
    struct LastSpeedChanged {
        previous: Speed,
    }

    fn record(event: On<SpeedChanged>, mut commands: Commands) {
        commands.insert_resource(LastSpeedChanged {
            previous: event.previous,
        });
    }

    #[test]
    fn should_emit_when_speed_changes() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddSpeedModifierPlugin, SyncSpeedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((BaseSpeed(220), Speed(220))).id();

        app.world_mut().trigger(AddSpeedModifierIntent {
            entity,
            modifier: SpeedModifier::new(30),
        });

        app.update();

        assert_eq!(
            app.world().resource::<LastSpeedChanged>().previous,
            Speed(220)
        );

        assert_eq!(
            **app
                .world()
                .get::<Speed>(entity)
                .expect("Speed should exist"),
            250
        );
    }

    #[test]
    fn should_not_emit_when_speed_does_not_change() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AddSpeedModifierPlugin, SyncSpeedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((BaseSpeed(220), Speed(220))).id();

        app.world_mut().trigger(AddSpeedModifierIntent {
            entity,
            modifier: SpeedModifier::new(0),
        });

        app.update();

        assert!(!app.world().contains_resource::<LastSpeedChanged>());
    }
}
