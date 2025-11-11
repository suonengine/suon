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
