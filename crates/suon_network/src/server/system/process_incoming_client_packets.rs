use bevy::prelude::*;

use crate::server::{connection::Connection, packet::Packet};

/// Processes all packets received from clients and forwards them
/// to the message writer for further handling.
pub(crate) fn process_incoming_client_packets(
    query: Query<(Entity, &Connection)>,
    mut packets: MessageWriter<Packet>,
) {
    for (client, connection) in query {
        // Send all transformed packets to the writer in a batch
        packets.write_batch(connection.read().into_iter().map(|incoming_packet| {
            trace!(
                "Forwarding packet from {} (client {:?}): kind={:?}",
                connection.addr(),
                client,
                incoming_packet.kind,
            );

            Packet {
                client,
                timestamp: incoming_packet.timestamp,
                checksum: incoming_packet.checksum,
                kind: incoming_packet.kind,
                buffer: incoming_packet.buffer,
            }
        }));
    }
}
