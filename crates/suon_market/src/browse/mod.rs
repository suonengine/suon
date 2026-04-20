mod events;
mod player;
mod scope;
mod session;

use bevy::prelude::*;
use log::warn;
use suon_movement::prelude::{Step, Teleport};
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::{BrowseMarketPacket, LeaveMarketPacket};

pub use self::{
    events::{MarketBrowse, MarketBrowseIntent, MarketBrowseRejected},
    player::MarketActorRef,
    scope::{MarketBrowseScope, TryMarketRequestKindFromPacketError},
    session::MarketSession,
};

pub(crate) struct MarketBrowsePlugin;

impl Plugin for MarketBrowsePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_leave_market)
            .add_observer(on_market_session_closed_by_step)
            .add_observer(on_market_session_closed_by_teleport)
            .add_observer(on_browse_market_packet)
            .add_observer(on_market_browse_intent);
    }
}

fn on_leave_market(
    event: On<Packet<LeaveMarketPacket>>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let client = event.entity();

    if sessions.contains(client) {
        commands.entity(client).remove::<MarketSession>();
    }
}

fn on_market_session_closed_by_step(
    event: On<Step>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let client = event.event_target();

    if sessions.contains(client) {
        commands.entity(client).remove::<MarketSession>();
    }
}

fn on_market_session_closed_by_teleport(
    event: On<Teleport>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let client = event.event_target();

    if sessions.contains(client) {
        commands.entity(client).remove::<MarketSession>();
    }
}

fn on_browse_market_packet(
    event: On<Packet<BrowseMarketPacket>>,
    mut commands: Commands,
    actor_refs: Query<&MarketActorRef>,
) {
    let client = event.entity();
    let packet = event.packet();

    let Ok(scope) = MarketBrowseScope::try_from(packet) else {
        warn!(
            "Ignoring malformed market browse packet from entity {:?}",
            client
        );

        commands.trigger(MarketBrowseRejected {
            client,
            request_kind: packet.request_kind,
            item_id: packet.sprite_id,
        });
        return;
    };

    let actor_id = actor_refs.get(client).ok().map(MarketActorRef::actor_id);
    commands.trigger(MarketBrowseIntent {
        client,
        actor_id,
        scope,
    });
}

fn on_market_browse_intent(event: On<MarketBrowseIntent>, mut commands: Commands) {
    commands
        .entity(event.client)
        .insert(MarketSession::new(Some(event.scope.clone())));

    commands.trigger(MarketBrowse {
        client: event.client,
        actor_id: event.actor_id,
        scope: event.scope.clone(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use suon_chunk::prelude::{Chunk, ChunkPlugin, Chunks};
    use suon_movement::prelude::{MovementPlugins, StepIntent, TeleportIntent};
    use suon_position::prelude::{Floor, Position};
    use suon_protocol_client::prelude::BrowseMarketPacket;

    #[test]
    fn should_translate_browse_scope_to_own_offers() {
        let packet = BrowseMarketPacket {
            request_kind: suon_protocol_client::prelude::MarketBrowseKind::OwnOffers,
            sprite_id: None,
        };

        assert_eq!(
            MarketBrowseScope::try_from(&packet),
            Ok(MarketBrowseScope::OwnOffers)
        );
    }

    #[test]
    fn should_translate_browse_scope_to_item() {
        let packet = BrowseMarketPacket {
            request_kind: suon_protocol_client::prelude::MarketBrowseKind::Item,
            sprite_id: Some(2160),
        };

        assert_eq!(
            MarketBrowseScope::try_from(&packet),
            Ok(MarketBrowseScope::Item { item_id: 2160 })
        );
    }

    #[test]
    fn should_reject_item_browse_without_item_id() {
        let packet = BrowseMarketPacket {
            request_kind: suon_protocol_client::prelude::MarketBrowseKind::Item,
            sprite_id: None,
        };

        assert_eq!(
            MarketBrowseScope::try_from(&packet),
            Err(TryMarketRequestKindFromPacketError::MissingItemId {
                request_kind: suon_protocol_client::prelude::MarketBrowseKind::Item,
            })
        );
    }

    #[test]
    fn should_close_market_session_when_step_event_is_received() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);
        app.add_plugins(MovementPlugins);
        app.add_plugins(MarketBrowsePlugin);

        const START: Position = Position { x: 10, y: 10 };
        const TARGET: Position = Position { x: 11, y: 10 };
        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(START, chunk), (TARGET, chunk)]));

        let client = app
            .world_mut()
            .spawn((
                MarketSession::new(Some(MarketBrowseScope::OwnOffers)),
                START,
                Floor { z: 0 },
            ))
            .id();

        app.world_mut().trigger(StepIntent {
            to: suon_position::prelude::Direction::East,
            entity: client,
        });
        app.update();

        assert!(
            app.world().get::<MarketSession>(client).is_none(),
            "A successful step should close the open market session"
        );
    }

    #[test]
    fn should_close_market_session_when_teleport_event_is_received() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);
        app.add_plugins(MovementPlugins);
        app.add_plugins(MarketBrowsePlugin);

        const START: Position = Position { x: 20, y: 20 };
        const TARGET: Position = Position { x: 25, y: 20 };
        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([
            (START, start_chunk),
            (TARGET, target_chunk),
        ]));

        let client = app
            .world_mut()
            .spawn((
                MarketSession::new(Some(MarketBrowseScope::OwnHistory)),
                START,
            ))
            .id();

        app.world_mut().trigger(TeleportIntent {
            to: TARGET,
            floor: None,
            entity: client,
        });
        app.update();
        app.update();

        assert!(
            app.world().get::<MarketSession>(client).is_none(),
            "A successful teleport should close the open market session"
        );
    }
}
