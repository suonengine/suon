use bevy::prelude::*;
use suon_protocol::packets::client::{
    Decodable, PacketKind,
    prelude::{KeepAlivePacket, PingLatencyPacket},
};

use crate::server::{
    connection::Connection,
    packet::{DecodeError, Packet},
};

macro_rules! dispatch_packet {
    ($commands:expr, $client:expr, $incoming_packet:expr; $( $packet_ty:ty ),* $(,)?) => {
        match $incoming_packet.kind {
            PacketKind::ServerName => {
                // First packet sent by the client during the connection handshake.
            }
            $(
                <$packet_ty>::KIND => {
                    (|| {
                        let incoming_packet = $incoming_packet;
                        let kind = incoming_packet.kind;
                        let timestamp = incoming_packet.timestamp;
                        let checksum = incoming_packet.checksum;
                        let mut bytes = incoming_packet.buffer.as_ref();

                        let packet = match <$packet_ty>::decode(&mut bytes) {
                            Ok(packet) => packet,
                            Err(err) => {
                                error!("Failed to decode packet for client {}: {:?}", $client, err);
                                let error = DecodeError::from(err);
                                warn!(
                                    "Failed to dispatch typed packet event for client {} and kind {:?}: {}",
                                    $client, kind, error
                                );
                                return None;
                            }
                        };

                        if !bytes.is_empty() {
                            let error = DecodeError::ExtraBytes(bytes.len());
                            warn!(
                                "Failed to dispatch typed packet event for client {} and kind {:?}: {}",
                                $client, kind, error
                            );
                            return None;
                        }

                        debug!("Successfully decoded packet for client {}", $client);

                        Some(Packet {
                            entity: $client,
                            timestamp,
                            checksum,
                            packet,
                        })
                    })()
                    .map(|packet| $commands.trigger(packet));
                }
            )*
            kind => {
                trace!(
                    "Skipping typed dispatch for client packet kind {:?} until a typed event \
                    is registered",
                    kind
                );
            }
        }
    };
}

/// Processes all packets received from clients and dispatches typed packet events.
pub(crate) fn process_incoming_client_packets(
    query: Query<(Entity, &Connection)>,
    mut commands: Commands,
) {
    for (client, connection) in query {
        for incoming_packet in connection.read() {
            trace!(
                "Dispatching packet from {} (client {:?}): kind={:?}",
                connection.addr(),
                client,
                incoming_packet.kind,
            );

            dispatch_packet!(
                commands,
                client,
                incoming_packet;
                KeepAlivePacket,
                PingLatencyPacket,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{
        connection::Connection,
        packet::{Packet, incoming::IncomingPacket},
        settings::PacketPolicy,
    };
    use bytes::Bytes;
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        time::Instant,
    };
    use suon_checksum::Adler32Checksum;
    use suon_protocol::packets::client::DecodableError;

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct ObservedPackets(Vec<&'static str>);

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct PingLatencyMeta(Option<(Entity, Instant, Option<Adler32Checksum>)>);

    #[derive(Debug)]
    struct FailingPacket;

    impl Decodable for FailingPacket {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
            Err(DecodableError::Decoder(
                suon_protocol::packets::decoder::DecoderError::Incomplete {
                    expected: 1,
                    available: 0,
                },
            ))
        }
    }

    fn observe_keep_alive(
        event: On<Packet<KeepAlivePacket>>,
        mut observed: ResMut<ObservedPackets>,
    ) {
        assert!(
            matches!(event.packet(), KeepAlivePacket),
            "The keep-alive observer should receive the decoded packet payload"
        );
        observed.0.push("keep_alive");
    }

    fn observe_ping_latency(
        event: On<Packet<PingLatencyPacket>>,
        mut observed: ResMut<ObservedPackets>,
        mut metadata: ResMut<PingLatencyMeta>,
    ) {
        assert!(
            matches!(event.packet(), PingLatencyPacket),
            "The ping-latency observer should receive the decoded packet payload"
        );
        observed.0.push("ping_latency");
        metadata.0 = Some((event.entity(), event.timestamp(), event.checksum()));
    }

    fn build_connection(packets: impl IntoIterator<Item = IncomingPacket>) -> Connection {
        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        for packet in packets {
            incoming_sender
                .send(packet)
                .expect("The incoming packet channel should accept the test packet");
        }

        Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        )
    }

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ObservedPackets>();
        app.init_resource::<PingLatencyMeta>();
        app.add_observer(observe_keep_alive);
        app.add_observer(observe_ping_latency);
        app.add_systems(Update, process_incoming_client_packets);
        app
    }

    #[test]
    fn should_dispatch_all_queued_incoming_packets_as_typed_events() {
        let mut app = build_app();

        let timestamp = Instant::now();
        let checksum = Adler32Checksum::from(0x1234_5678_u32);
        let connection = build_connection([IncomingPacket {
            timestamp,
            checksum: Some(checksum),
            kind: PacketKind::PingLatency,
            buffer: Bytes::new(),
        }]);

        let client = app.world_mut().spawn(connection).id();

        app.update();
        app.update();

        let observed = app.world().resource::<ObservedPackets>();
        let metadata = app.world().resource::<PingLatencyMeta>();

        assert_eq!(
            observed.0,
            vec!["ping_latency"],
            "process_incoming_client_packets should emit one typed event per queued incoming \
             packet"
        );
        assert_eq!(
            metadata.0,
            Some((client, timestamp, Some(checksum))),
            "typed packet dispatch should preserve the original packet metadata"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_connections_have_no_incoming_packets() {
        let mut app = build_app();

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
        app.update();

        let messages = app.world().resource::<ObservedPackets>();
        assert_eq!(
            messages.0.len(),
            0,
            "process_incoming_client_packets should skip connections without queued packets"
        );
    }

    #[test]
    fn should_dispatch_every_packet_currently_queued_on_the_connection_in_order() {
        let mut app = build_app();

        let connection = build_connection([
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
                buffer: Bytes::new(),
            },
        ]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        let observed = app.world().resource::<ObservedPackets>();

        assert_eq!(
            observed.0,
            vec!["keep_alive", "ping_latency"],
            "process_incoming_client_packets should dispatch all packets currently queued in \
             receive order"
        );
    }

    #[test]
    fn should_ignore_server_name_packets_during_dispatch() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::ServerName,
            buffer: Bytes::from_static(b"otserv\n"),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert!(
            app.world().resource::<ObservedPackets>().0.is_empty(),
            "ServerName packets should be ignored by the typed packet dispatcher"
        );
    }

    #[test]
    fn should_ignore_kinds_without_registered_typed_dispatch() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::Login,
            buffer: Bytes::new(),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert!(
            app.world().resource::<ObservedPackets>().0.is_empty(),
            "Unregistered packet kinds should not emit typed packet events"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_payload_has_extra_bytes() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::KeepAlive,
            buffer: Bytes::from_static(&[1]),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert!(
            app.world().resource::<ObservedPackets>().0.is_empty(),
            "Packets that leave unread payload bytes should be rejected during typed dispatch"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_decoding_fails() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, |mut commands: Commands| {
            dispatch_packet!(
                commands,
                Entity::from_bits(7),
                IncomingPacket {
                    timestamp: Instant::now(),
                    checksum: None,
                    kind: PacketKind::PingLatency,
                    buffer: Bytes::new(),
                };
                FailingPacket,
            );
        });

        #[derive(Resource, Default)]
        struct Triggered(bool);

        fn observe_failing_packet(
            _event: On<Packet<FailingPacket>>,
            mut triggered: ResMut<Triggered>,
        ) {
            triggered.0 = true;
        }

        app.init_resource::<Triggered>();
        app.add_observer(observe_failing_packet);

        app.update();
        app.update();

        assert!(
            !app.world().resource::<Triggered>().0,
            "Failed packet decodes should not emit typed packet events"
        );
    }
}
