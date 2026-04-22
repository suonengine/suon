use bevy::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::BrowseMarketPacket;

use crate::browse::BrowseMarketIntent;

/// Translates inbound market-browse packets into typed market intents.
pub(super) fn on_browse_market_packet(
    event: On<Packet<BrowseMarketPacket>>,
    mut commands: Commands,
) {
    let entity = event.entity();
    let scope = event.packet().request_kind;

    commands.trigger(BrowseMarketIntent { entity, scope });
}
