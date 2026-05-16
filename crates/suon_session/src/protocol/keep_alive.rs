use bevy::prelude::*;
use suon_network::prelude::*;
use suon_protocol_client::prelude::KeepAlivePacket as KeepAliveRequest;
use suon_protocol_server::prelude::KeepAlivePacket as KeepAliveResponse;

pub(super) fn on_keep_alive(
    event: On<Packet<KeepAliveRequest>>,
    connections: Query<&Connection>,
) {
    let entity = event.entity();
    if let Ok(connection) = connections.get(entity) {
        let _ = connection.write(KeepAliveResponse);
    }
}
