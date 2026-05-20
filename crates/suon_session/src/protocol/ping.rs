use bevy::prelude::*;
use suon_network::prelude::*;
use suon_protocol_client::prelude::PingLatencyPacket as PingRequest;
use suon_protocol_server::prelude::PingLatencyPacket as PingResponse;

pub(super) fn on_ping(event: On<Packet<PingRequest>>, connections: Query<&Connection>) {
    let entity = event.entity();
    if let Ok(connection) = connections.get(entity) {
        let _ = connection.write(PingResponse);
    }
}
