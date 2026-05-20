use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use suon_network::prelude::*;
use suon_protocol_client::prelude::ServerNamePacket;
use suon_protocol_server::prelude::ChallengePacket;

pub(super) fn on_server_name(event: On<Packet<ServerNamePacket>>, connections: Query<&Connection>) {
    let entity = event.entity();
    let Ok(connection) = connections.get(entity) else {
        return;
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let challenge = ChallengePacket {
        timestamp: SystemTime::now(),
        random_number: (now & 0xFF) as u8,
    };

    let _ = connection.write(challenge);
}
