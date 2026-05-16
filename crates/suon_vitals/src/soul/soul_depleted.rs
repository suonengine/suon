use bevy::prelude::*;
use log::trace;
use suon_macros::*;

use super::{consume_soul::ConsumeSoul, *};

/// Event emitted when soul reaches zero.
#[derive(EntityEvent, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SoulDepleted {
    /// Entity whose soul reached zero.
    #[event_target]
    entity: Entity,
}

pub(crate) struct SoulDepletedPlugin;

impl Plugin for SoulDepletedPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(emit_soul_depleted_after_consume);
    }
}

fn emit_soul_depleted_after_consume(
    event: On<ConsumeSoul>,
    mut commands: Commands,
    current: Query<&Soul>,
) {
    let entity = event.event_target();

    let Ok(current) = current.get(entity) else {
        debug_unreachable!("SoulDepleted observer received {entity:?} without Soul");
        return;
    };

    let soul_depleted = !event.previous.is_zero() && current.is_zero();
    if soul_depleted {
        trace!("Emitting SoulDepleted for {entity:?}");

        commands.trigger(SoulDepleted { entity });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::soul::consume_soul::{ConsumeSoulIntent, ConsumeSoulPlugin};

    #[derive(Resource)]
    struct LastSoulDepleted {
        entity: Entity,
    }

    fn record(event: On<SoulDepleted>, mut commands: Commands) {
        commands.insert_resource(LastSoulDepleted {
            entity: event.event_target(),
        });
    }

    #[test]
    fn should_emit_when_soul_reaches_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin, SoulDepletedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(50), MaxSoul(100))).id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 50 });

        app.update();

        assert_eq!(app.world().resource::<LastSoulDepleted>().entity, entity);
    }

    #[test]
    fn should_not_emit_when_soul_does_not_reach_zero() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ConsumeSoulPlugin, SoulDepletedPlugin));
        app.add_observer(record);

        let entity = app.world_mut().spawn((Soul(100), MaxSoul(100))).id();

        app.world_mut()
            .trigger(ConsumeSoulIntent { entity, amount: 50 });

        app.update();

        assert!(!app.world().contains_resource::<LastSoulDepleted>());
    }
}
