use bevy::{prelude::*, tasks::IoTaskPool};
use humantime::format_duration;
use smol::io::AsyncWriteExt;
use smol_timeout::TimeoutExt;
use std::net::SocketAddr;
use suon_xtea::XTEAKey;

use crate::server::{
    connection::{Connection, incoming::IncomingConnections, outgoing::OutgoingConnections},
    packet::{
        incoming::{
            IncomingPacket, login::LoginReadPacket, server_name::ServerNameReadPacketExt,
            subsequent::SubsequentReadPacket,
        },
        outgoing::OutgoingPacket,
    },
    settings::{IncomingPacketPolicy, OutgoingPacketPolicy, Settings},
};

/// Processes new incoming client connections.
///
/// Spawns a new Bevy entity for each client, sets up reader and writer tasks,
/// and channels for handling incoming and outgoing packets.
pub(crate) fn accept_client_connections(
    mut commands: Commands,
    incoming_connections: Res<IncomingConnections>,
    outgoing_connections: Res<OutgoingConnections>,
    settings: Res<Settings>,
) {
    for stream in incoming_connections.read() {
        let Ok(addr) = stream.peer_addr() else {
            warn!("Failed to get peer address for incoming connection, skipping.");
            continue;
        };

        info!("Accepting new client connection from {}", addr);

        // Create an entity representing this client connection
        let client = commands.spawn_empty().id();
        debug!("Spawned new entity {:?} for client {}", client, addr);

        // Channel for sending decoded incoming packets from reader task
        let (incoming_packet_sender, incoming_packet_receiver) =
            crossbeam_channel::unbounded::<IncomingPacket>();

        // Channel for sending outgoing packets to writer task
        let (outgoing_packet_sender, outgoing_packet_receiver) =
            crossbeam_channel::unbounded::<OutgoingPacket>();

        // Watch channel for optional XTEA encryption key, allows runtime updates
        let (xtea_key_sender, xtea_key_receiver) = tokio::sync::watch::channel(None);

        // Spawn asynchronous writer task responsible for sending packets to the client
        spawn_writer_task(
            stream.clone(),
            addr,
            client,
            outgoing_packet_receiver,
            outgoing_connections.clone(),
            settings.packet_policy.outgoing,
        );

        debug!("Spawned writer task for client {}", addr);

        // Spawn asynchronous reader task responsible for decoding incoming packets
        spawn_reader_task(
            stream.clone(),
            addr,
            client,
            incoming_packet_sender,
            outgoing_connections.clone(),
            xtea_key_receiver,
            settings.packet_policy.incoming,
        );

        debug!("Spawned reader task for client {}", addr);

        commands.entity(client).insert(Connection::new(
            outgoing_packet_sender,
            incoming_packet_receiver,
            addr,
            xtea_key_sender,
            settings.packet_policy,
        ));

        info!(
            "Connection setup completed for client {} (entity {:?})",
            addr, client
        );
    }
}

/// Spawns an asynchronous task responsible for sending outgoing packets to a client.
fn spawn_writer_task(
    mut stream: smol::net::TcpStream,
    addr: SocketAddr,
    client: Entity,
    outgoing_packet_receiver: crossbeam_channel::Receiver<OutgoingPacket>,
    outgoing_connections: OutgoingConnections,
    outgoing_packet_policy: OutgoingPacketPolicy,
) {
    IoTaskPool::get()
        .spawn(async move {
            info!("Writer task started for client {:?} at {}", client, addr);

            // Process outgoing packets as they arrive on the channel...
            while let Ok(packet) = outgoing_packet_receiver.recv() {
                // Encode the packet into bytes for transmission
                let encoded_bytes = packet.encode();

                trace!(
                    "Preparing to send packet for client {client} at {addr} with {} bytes",
                    encoded_bytes.len()
                );

                // Attempt to write the encoded packet to the stream
                match stream
                    .write_all(&encoded_bytes)
                    .timeout(outgoing_packet_policy.timeout)
                    .await
                    .transpose()
                {
                    Ok(Some(..)) => trace!(
                        "Successfully wrote packet to client {client} at {addr} with {} bytes",
                        encoded_bytes.len()
                    ),
                    Ok(None) => {
                        warn!(
                            "Write timeout for client {client} at {addr} after {}",
                            format_duration(outgoing_packet_policy.timeout)
                        );
                        break;
                    }
                    Err(err) => {
                        error!("Write error for client {client} at {addr}: {}", err);
                        break;
                    }
                }

                // Flush the stream to ensure all data is sent
                match stream
                    .flush()
                    .timeout(outgoing_packet_policy.timeout)
                    .await
                    .transpose()
                {
                    Ok(Some(..)) => {
                        trace!("Stream flushed successfully for client {client} at {addr}")
                    }
                    Ok(None) => warn!(
                        "Flush timeout for client {client} at {addr} after {}",
                        format_duration(outgoing_packet_policy.timeout)
                    ),
                    Err(err) => warn!("Flush error for client {client} at {addr}: {}", err),
                }
            }

            if let Err(err) = outgoing_connections.send((client, addr)) {
                warn!("Failed to enqueue outgoing connection for client {client} at {addr}: {err}");
            }

            // This point is reached when the channel is closed or an error occurs
            info!("Writer task closed for client {:?} at {}", client, addr);
        })
        .detach();
}

/// Spawns an asynchronous task responsible for reading and decoding incoming packets from a client.
fn spawn_reader_task(
    mut stream: smol::net::TcpStream,
    addr: SocketAddr,
    client: Entity,
    incoming_packet_sender: crossbeam_channel::Sender<IncomingPacket>,
    outgoing_connections: OutgoingConnections,
    mut xtea_key_receiver: tokio::sync::watch::Receiver<Option<XTEAKey>>,
    incoming_packet_policy: IncomingPacketPolicy,
) {
    IoTaskPool::get()
        .spawn(async move {
            info!("Reader task started for client {:?} at {}", client, addr);

            // Attempt to read the server name packet
            match stream
                .read_server_name_packet(incoming_packet_policy.server_name_max_length)
                .timeout(incoming_packet_policy.timeout)
                .await
                .transpose()
            {
                Ok(Some(packet)) => {
                    trace!(
                        "Server name packet received and forwarded for client {client} at {addr}",
                    );

                    incoming_packet_sender.send(packet).ok();
                }
                Ok(None) => {
                    warn!(
                        "Timeout while reading server name packet for client {client} at {addr} \
                         after {}",
                        format_duration(incoming_packet_policy.timeout)
                    );

                    outgoing_connections.send((client, addr)).ok();
                    return;
                }
                Err(err) => {
                    info!(
                        "Reader task ending while reading login packet for client {client} at \
                         {addr}: {err}"
                    );

                    outgoing_connections.send((client, addr)).ok();
                    return;
                }
            }

            // Attempt to read the login packet
            match stream
                .read_login_packet(incoming_packet_policy.login_max_length)
                .timeout(incoming_packet_policy.timeout)
                .await
                .transpose()
            {
                Ok(Some(packet)) => {
                    trace!("Login packet received and forwarded for client {client} at {addr}");

                    incoming_packet_sender.send(packet).ok();
                }
                Ok(None) => {
                    warn!(
                        "Timeout while reading login packet for client {client} at {addr} after {}",
                        format_duration(incoming_packet_policy.timeout)
                    );

                    outgoing_connections.send((client, addr)).ok();
                    return;
                }
                Err(err) => {
                    info!(
                        "Reader task ending while reading login packet for client {client} at \
                         {addr}: {err}"
                    );

                    outgoing_connections.send((client, addr)).ok();
                    return;
                }
            }

            loop {
                // Wait for XTEA key
                if xtea_key_receiver
                    .changed()
                    .timeout(incoming_packet_policy.timeout)
                    .await
                    .is_none()
                {
                    warn!("Timeout waiting for XTEA key update, ending subsequent packet reader");
                    break;
                }

                let Some(xtea_key) = *xtea_key_receiver.borrow() else {
                    trace!("No XTEA key set yet, skipping subsequent packet read...");
                    break;
                };

                // Attempt to read the subsequent packet
                match stream
                    .read_subsequent_packet(xtea_key, incoming_packet_policy.subsequent_max_length)
                    .timeout(incoming_packet_policy.timeout)
                    .await
                    .transpose()
                {
                    Ok(Some(packet)) => {
                        trace!(
                            "Subsequent packet received and forwarded for client {client} at \
                             {addr}",
                        );

                        incoming_packet_sender.send(packet).ok();
                    }
                    Ok(None) => {
                        warn!(
                            "Timeout while reading subsequent packet for client {client} at \
                             {addr} after {}",
                            format_duration(incoming_packet_policy.timeout)
                        );
                        break;
                    }
                    Err(err) => {
                        info!(
                            "Reader task ending while reading lsubsequentogin packet for client \
                             {client} at {addr}: {err}"
                        );
                        break;
                    }
                }
            }

            outgoing_connections.send((client, addr)).ok();
        })
        .detach();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{app::App, tasks::TaskPool};
    use bytes::Bytes;
    use smol::io::AsyncReadExt;
    use std::{
        net::SocketAddr,
        thread,
        time::{Duration, Instant},
    };
    use suon_protocol_client::prelude::*;

    const XTEA_KEY: XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];

    fn init_io_task_pool() {
        IoTaskPool::get_or_init(TaskPool::new);
    }

    fn test_settings() -> Settings {
        Settings {
            packet_policy: crate::server::settings::PacketPolicy {
                incoming: IncomingPacketPolicy {
                    timeout: Duration::from_millis(100),
                    server_name_max_length: 256,
                    login_max_length: 256,
                    subsequent_max_length: 256,
                    ..IncomingPacketPolicy::default()
                },
                outgoing: OutgoingPacketPolicy {
                    timeout: Duration::from_millis(100),
                    max_length: 256,
                },
            },
            ..Settings::default()
        }
    }

    fn wait_for<T>(mut f: impl FnMut() -> Option<T>) -> T {
        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            if let Some(value) = f() {
                return value;
            }

            assert!(
                Instant::now() < deadline,
                "timed out while waiting for async network task to produce a result"
            );

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn build_login_packet_bytes(payload: &[u8], checksum: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(7 + payload.len());
        bytes.extend_from_slice(
            &((crate::server::packet::PACKET_CHECKSUM_SIZE + 1 + payload.len()) as u16)
                .to_le_bytes(),
        );
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes.push(PacketKind::Login as u8);
        bytes.extend_from_slice(payload);
        bytes
    }

    fn build_subsequent_packet_bytes(payload: &[u8], checksum: u32) -> Vec<u8> {
        let mut plaintext =
            Vec::with_capacity(crate::server::packet::PACKET_HEADER_SIZE + payload.len());
        plaintext.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        plaintext.extend_from_slice(payload);

        let encrypted = suon_xtea::encrypt(&plaintext, &XTEA_KEY);
        let mut bytes = Vec::with_capacity(
            crate::server::packet::PACKET_HEADER_SIZE
                + crate::server::packet::PACKET_CHECKSUM_SIZE
                + encrypted.len(),
        );
        bytes.extend_from_slice(
            &((crate::server::packet::PACKET_CHECKSUM_SIZE + encrypted.len()) as u16).to_le_bytes(),
        );
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes.extend_from_slice(&encrypted);
        bytes
    }

    fn connected_streams() -> (smol::net::TcpStream, smol::net::TcpStream, SocketAddr) {
        smol::block_on(async {
            let listener = smol::net::TcpListener::bind(("127.0.0.1", 0))
                .await
                .expect("the test listener should bind successfully");

            let addr = listener
                .local_addr()
                .expect("the test listener should expose a local address");

            let accept_task = smol::spawn(async move {
                listener
                    .accept()
                    .await
                    .expect("the test listener should accept one connection")
                    .0
            });

            let client = smol::net::TcpStream::connect(addr)
                .await
                .expect("the test client should connect successfully");

            let server = accept_task.await;

            let peer_addr = client
                .local_addr()
                .expect("the test client should expose its local address");

            (server, client, peer_addr)
        })
    }

    #[test]
    fn should_spawn_a_connection_entity_for_each_queued_incoming_stream() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<IncomingConnections>();
        app.init_resource::<OutgoingConnections>();
        app.insert_resource(test_settings());
        app.add_systems(Update, accept_client_connections);

        let (server_stream, client_stream, client_addr) = connected_streams();
        app.world()
            .resource::<IncomingConnections>()
            .send(server_stream)
            .expect("the incoming queue should accept the test connection");

        app.update();

        let mut connections = app.world_mut().query::<&Connection>();

        let connection = connections
            .iter(app.world())
            .next()
            .expect("accept_client_connections should spawn one connection component");

        assert_eq!(
            connection.addr(),
            client_addr,
            "the spawned connection should keep the peer address from the accepted socket"
        );

        drop(client_stream);
    }

    #[test]
    fn should_write_encoded_packets_and_mark_the_connection_as_finished_when_the_channel_closes() {
        init_io_task_pool();

        let (server_stream, mut client_stream, client_addr) = connected_streams();
        let client = Entity::from_bits(11);
        let outgoing_connections = OutgoingConnections::default();
        let (outgoing_sender, outgoing_receiver) = crossbeam_channel::unbounded();
        let expected =
            crate::server::packet::outgoing::OutgoingPacket::new(Bytes::from_static(b"\x01\x02"))
                .encode();

        spawn_writer_task(
            server_stream,
            client_addr,
            client,
            outgoing_receiver,
            outgoing_connections.clone(),
            test_settings().packet_policy.outgoing,
        );

        outgoing_sender
            .send(crate::server::packet::outgoing::OutgoingPacket::new(
                Bytes::from_static(b"\x01\x02"),
            ))
            .expect("the writer task should accept one outgoing packet");

        drop(outgoing_sender);

        let mut encoded = vec![0; expected.len()];
        smol::block_on(async {
            client_stream
                .read_exact(&mut encoded)
                .await
                .expect("the client side should receive the encoded packet bytes");
        });

        assert_eq!(
            encoded, expected,
            "spawn_writer_task should write the packet bytes exactly as encoded"
        );

        let closed = wait_for(|| outgoing_connections.read().into_iter().next());
        assert_eq!(
            closed,
            (client, client_addr),
            "spawn_writer_task should enqueue the closed connection after the sender is dropped"
        );
    }

    #[test]
    fn should_forward_initial_and_subsequent_packets_then_close_the_connection() {
        init_io_task_pool();

        let (server_stream, mut client_stream, client_addr) = connected_streams();
        let client = Entity::from_bits(22);
        let outgoing_connections = OutgoingConnections::default();
        let (incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, xtea_receiver) = tokio::sync::watch::channel(None);

        spawn_reader_task(
            server_stream,
            client_addr,
            client,
            incoming_sender,
            outgoing_connections.clone(),
            xtea_receiver,
            test_settings().packet_policy.incoming,
        );

        smol::block_on(async {
            use smol::io::AsyncWriteExt;

            client_stream
                .write_all(b"suon\n")
                .await
                .expect("the client should send the server-name packet");

            client_stream
                .flush()
                .await
                .expect("the server-name packet should be flushed to the socket");
        });

        let server_name = wait_for(|| incoming_receiver.try_recv().ok());
        assert_eq!(
            server_name.kind,
            PacketKind::ServerName,
            "spawn_reader_task should forward the first server-name packet"
        );

        smol::block_on(async {
            use smol::io::AsyncWriteExt;

            client_stream
                .write_all(&build_login_packet_bytes(b"login", 0))
                .await
                .expect("the client should send the login packet");

            client_stream
                .flush()
                .await
                .expect("the login packet should be flushed to the socket");
        });

        let login = wait_for(|| incoming_receiver.try_recv().ok());
        assert_eq!(
            login.kind,
            PacketKind::Login,
            "spawn_reader_task should forward the login packet after the server name"
        );

        xtea_sender
            .send(Some(XTEA_KEY))
            .expect("the test should be able to publish an XTEA key");

        smol::block_on(async {
            use smol::io::AsyncWriteExt;

            client_stream
                .write_all(&build_subsequent_packet_bytes(
                    &[PacketKind::PingLatency as u8, 1, 2],
                    0,
                ))
                .await
                .expect("the client should send one encrypted subsequent packet");

            client_stream
                .flush()
                .await
                .expect("the subsequent packet should be flushed to the socket");
        });

        let subsequent = wait_for(|| incoming_receiver.try_recv().ok());
        assert_eq!(
            subsequent.kind,
            PacketKind::PingLatency,
            "spawn_reader_task should decode and forward encrypted subsequent packets"
        );

        assert_eq!(
            subsequent.buffer.as_ref(),
            &[1, 2],
            "spawn_reader_task should preserve the decrypted subsequent payload"
        );

        drop(xtea_sender);
        drop(client_stream);

        let closed = wait_for(|| outgoing_connections.read().into_iter().next());
        assert_eq!(
            closed,
            (client, client_addr),
            "spawn_reader_task should notify the finished connection when the stream ends"
        );
    }
}
