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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{
        connection::Connection, packet::incoming::IncomingPacket, settings::PacketPolicy,
    };
    use bevy::ecs::message::Messages;
    use bytes::Bytes;
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        time::Instant,
    };
    use suon_checksum::Adler32Checksum;
    use suon_protocol::packets::client::PacketKind;

    fn build_connection(packet: IncomingPacket) -> Connection {
        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        incoming_sender
            .send(packet)
            .expect("The incoming packet channel should accept the test packet");

        Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        )
    }

    #[test]
    fn should_forward_all_queued_incoming_packets_as_messages() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Packet>();
        app.add_systems(Update, process_incoming_client_packets);

        let timestamp = Instant::now();
        let checksum = Adler32Checksum::from(0x1234_5678_u32);
        let connection = build_connection(IncomingPacket {
            timestamp,
            checksum: Some(checksum),
            kind: PacketKind::PingLatency,
            buffer: Bytes::from_static(&[1, 2, 3]),
        });

        let client = app.world_mut().spawn(connection).id();

        app.update();

        let messages = app.world().resource::<Messages<Packet>>();
        let forwarded = messages
            .iter_current_update_messages()
            .collect::<Vec<&Packet>>();

        assert_eq!(
            forwarded.len(),
            1,
            "process_incoming_client_packets should emit one message per queued incoming packet"
        );
        assert_eq!(
            forwarded[0].client(),
            client,
            "Forwarded packet messages should preserve the originating client entity"
        );
        assert_eq!(
            forwarded[0].timestamp(),
            timestamp,
            "Forwarded packet messages should preserve the original timestamp"
        );
        assert_eq!(
            forwarded[0].checksum(),
            Some(checksum),
            "Forwarded packet messages should preserve the incoming checksum"
        );
        assert_eq!(
            forwarded[0].kind,
            PacketKind::PingLatency,
            "Forwarded packet messages should preserve the incoming packet kind"
        );
        assert_eq!(
            forwarded[0].buffer,
            Bytes::from_static(&[1, 2, 3]),
            "Forwarded packet messages should preserve the incoming payload"
        );
    }

    #[test]
    fn should_not_emit_messages_when_connections_have_no_incoming_packets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Packet>();
        app.add_systems(Update, process_incoming_client_packets);

        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (_incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        let connection = Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        );

        app.world_mut().spawn(connection);
        app.update();

        let messages = app.world().resource::<Messages<Packet>>();
        assert_eq!(
            messages.iter_current_update_messages().count(),
            0,
            "process_incoming_client_packets should skip connections without queued packets"
        );
    }

    #[test]
    fn should_forward_every_packet_currently_queued_on_the_connection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<Packet>();
        app.add_systems(Update, process_incoming_client_packets);

        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        let packets = [
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::KeepAlive,
                buffer: Bytes::from_static(&[]),
            },
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::PingLatency,
                buffer: Bytes::from_static(&[7, 8]),
            },
        ];

        for packet in packets {
            incoming_sender
                .send(packet)
                .expect("The incoming channel should accept all queued test packets");
        }

        let connection = Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        );

        app.world_mut().spawn(connection);
        app.update();

        let messages = app.world().resource::<Messages<Packet>>();
        let forwarded = messages
            .iter_current_update_messages()
            .collect::<Vec<&Packet>>();

        assert_eq!(
            forwarded.len(),
            2,
            "process_incoming_client_packets should forward all packets currently queued"
        );
        assert_eq!(
            forwarded[0].kind,
            PacketKind::KeepAlive,
            "The first forwarded message should preserve the first queued packet kind"
        );
        assert_eq!(
            forwarded[1].kind,
            PacketKind::PingLatency,
            "The second forwarded message should preserve the second queued packet kind"
        );
    }
}
