use bevy::prelude::*;
use suon_movement::prelude::{Step, Teleport};
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::LeaveMarketPacket;

use crate::browse::MarketSession;

/// Removes the open market session when the client explicitly leaves the market.
pub(super) fn on_leave_market_packet(
    event: On<Packet<LeaveMarketPacket>>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let entity = event.entity();

    if sessions.contains(entity) {
        commands.entity(entity).remove::<MarketSession>();
    }
}

/// Removes the open market session after a successful step.
pub(super) fn on_step_close_market_session(
    event: On<Step>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let entity = event.event_target();

    if sessions.contains(entity) {
        commands.entity(entity).remove::<MarketSession>();
    }
}

/// Removes the open market session after a successful teleport.
pub(super) fn on_teleport_close_market_session(
    event: On<Teleport>,
    mut commands: Commands,
    sessions: Query<(), With<MarketSession>>,
) {
    let entity = event.event_target();

    if sessions.contains(entity) {
        commands.entity(entity).remove::<MarketSession>();
    }
}
