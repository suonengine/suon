use bevy::prelude::*;

use crate::server::connection::Connection;

/// Flushes the outgoing packet buffers of all active client connections.
///
/// This function iterates over all entities with a `Connection` component and attempts
/// to send any buffered outgoing data.
pub(crate) fn flush_connection_buffers(query: Query<(Entity, &Connection)>) {
    for (client, connection) in query {
        // Attempt to flush the buffer for this connection
        if let Some(flushed_bytes) = connection.flush() {
            debug!(
                "Flushed {} bytes from outgoing buffer of client {} (entity {:?})",
                flushed_bytes,
                connection.addr(),
                client
            );

            trace!(
                "Connection flush completed for client {} (entity {:?})",
                connection.addr(),
                client
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{connection::Connection, settings::PacketPolicy};
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use suon_protocol::packets::server::{Encodable, PacketKind};

    struct DummyPacket;

    impl Encodable for DummyPacket {
        const KIND: PacketKind = PacketKind::KeepAlive;
    }

    fn build_connection() -> (
        Connection,
        crossbeam_channel::Receiver<crate::server::packet::outgoing::OutgoingPacket>,
    ) {
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::unbounded();
        let (_incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        let connection = Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        );

        (connection, outgoing_receiver)
    }

    #[test]
    fn should_flush_buffered_packets_for_active_connections() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, flush_connection_buffers);

        let (connection, outgoing_receiver) = build_connection();
        connection
            .write(DummyPacket)
            .expect("Writing a small packet should populate the connection buffer");

        app.world_mut().spawn(connection);
        app.update();

        let packet = outgoing_receiver
            .try_recv()
            .expect("flush_connection_buffers should emit one outgoing packet for buffered data");

        assert!(
            !packet.encode().is_empty(),
            "The flushed packet should contain encoded bytes ready for transmission"
        );
    }

    #[test]
    fn should_not_emit_packets_when_the_connection_buffer_is_empty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, flush_connection_buffers);

        let (connection, outgoing_receiver) = build_connection();

        app.world_mut().spawn(connection);
        app.update();

        assert!(
            outgoing_receiver.try_recv().is_err(),
            "flush_connection_buffers should not emit packets for empty buffers"
        );
    }
}
