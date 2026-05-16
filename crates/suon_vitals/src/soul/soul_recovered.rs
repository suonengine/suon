use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{restore_soul::RestoreSoul, *};

/// Event emitted when soul rises from zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SoulRecovered {
    /// Entity whose soul rose from zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct SoulRecoveredPlugin;

impl Plugin for SoulRecoveredPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_soul_recovered_after_restore);
    }
}

fn emit_soul_recovered_after_restore(
    event: On<RestoreSoul>,
    mut commands: Commands,
    current: Query<&Soul>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("SoulRecovered observer received {entity:?} without Soul");
        return;
    };

    let soul_recovered = event.previous.is_zero() && !current.is_zero();
    if soul_recovered {
        trace!("Emitting SoulRecovered for {entity:?}");

        commands.trigger(SoulRecovered { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soul::restore_soul::{RestoreSoulIntent, RestoreSoulPlugin};

    #[derive(Resource)]
    struct LastSoulRecovered {
        entity: Entity,
    }

    fn record(event: On<SoulRecovered>, mut commands: Commands) {
        commands.insert_resource(LastSoulRecovered {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_soul_rises_from_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin, SoulRecoveredPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(0), MaxSoul(100))).id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 1 });

        app.update();

        assert_eq!(app.world().resource::<LastSoulRecovered>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_soul_was_already_above_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RestoreSoulPlugin, SoulRecoveredPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(50), MaxSoul(100))).id();

        app.world_mut()
            .trigger(RestoreSoulIntent { entity, amount: 25 });

        app.update();

        assert!(!app.world().contains_resource::<LastSoulRecovered>());
    }
}
