use bevy::prelude::*;
use bytes::BytesMut;
use std::{net::SocketAddr, sync::Mutex};
use suon_protocol::packets::server::Encodable;
use suon_xtea::XTEAKey;
use thiserror::Error;

use crate::server::{
    connection::checksum_mode::ChecksumMode,
    packet::{incoming::IncomingPacket, outgoing::OutgoingPacket},
    settings::PacketPolicy,
};

pub mod checksum_mode;
pub mod incoming;
pub mod limiter;
pub mod outgoing;
pub mod throttle;

/// Errors that can occur while writing or encoding packets.
///
/// These errors represent issues encountered during packet serialization,
/// buffering, or concurrent access to shared state. They typically occur when
/// packet size constraints are violated or when the writer lock cannot be acquired.
#[derive(Debug, Error)]
pub enum WriteError {
    /// The packet length exceeded the maximum allowed size.
    ///
    /// This usually indicates a malformed or oversized payload being written
    /// to the output buffer, exceeding protocol or implementation limits.
    #[error("packet length ({packet_len} bytes) exceeds the maximum allowed size")]
    Exceed {
        /// The actual length of the packet that triggered the error.
        packet_len: usize,
    },

    /// Failed to acquire the lock protecting the internal writer state.
    ///
    /// This may occur if another thread or task holds the writer lock for too
    /// long, or if the lock has been poisoned due to a previous panic.
    #[error("failed to acquire the internal writer lock")]
    LockFailed,
}

/// Represents a network connection to a client.
#[derive(Component)]
pub struct Connection {
    /// Channel used to send fully assembled packets to the writer task.
    sender: crossbeam_channel::Sender<OutgoingPacket>,

    /// Channel used to receive packets that arrived from the network.
    receiver: crossbeam_channel::Receiver<IncomingPacket>,

    /// The remote socket address associated with this connection.
    addr: SocketAddr,

    /// Buffer for assembling outgoing packets before sending them as a single chunk.
    buffer: Mutex<BytesMut>,

    /// Optional XTEA keys for encrypting outgoing packets.
    xtea_key: Option<XTEAKey>,
    xtea_key_shared: tokio::sync::watch::Sender<Option<XTEAKey>>,

    /// Optional checksum mode applied to outgoing packets.
    checksum_mode: Option<ChecksumMode>,

    /// Policy controlling packet sizes, flood protection and timing limits.
    packet_policy: PacketPolicy,
}

impl Connection {
    pub(crate) fn new(
        sender: crossbeam_channel::Sender<OutgoingPacket>,
        receiver: crossbeam_channel::Receiver<IncomingPacket>,
        addr: SocketAddr,
        xtea_key: tokio::sync::watch::Sender<Option<XTEAKey>>,
        packet_policy: PacketPolicy,
    ) -> Self {
        Self {
            sender,
            receiver,
            addr,
            buffer: Mutex::new(BytesMut::with_capacity(
                packet_policy.incoming.subsequent_max_length,
            )),
            xtea_key: None,
            xtea_key_shared: xtea_key,
            checksum_mode: None,
            packet_policy,
        }
    }

    /// Retrieves all currently queued incoming packets without blocking.
    pub(crate) fn read(&self) -> Vec<IncomingPacket> {
        self.receiver.try_iter().collect::<Vec<IncomingPacket>>()
    }

    /// Sends an `EncodablePacket` through the connection's writer.
    ///
    /// Returns the number of bytes written on success, or a `WriteError` if writing fails.
    pub fn write<P: Encodable>(&self, packet: P) -> Result<usize, WriteError> {
        let encoded_packet = packet.encode_with_kind();
        let encoded_packet_len = encoded_packet.len();

        // Reject packets that exceed the maximum packet length to avoid buffer overflow.
        if encoded_packet_len > self.packet_policy.outgoing.max_length {
            error!(
                "Packet length {encoded_packet_len} exceeds maximum allowed size of {}",
                self.packet_policy.outgoing.max_length
            );

            return Err(WriteError::Exceed {
                packet_len: encoded_packet_len,
            });
        }

        let mut buffer = match self.buffer.lock() {
            Ok(buffer) => buffer,
            Err(..) => {
                error!("Failed to acquire lock on connection buffer");
                return Err(WriteError::LockFailed);
            }
        };

        // If the new packet would overflow the buffer, flush existing data first.
        if buffer.len() + encoded_packet_len > self.packet_policy.outgoing.max_length {
            info!(
                "Buffer overflow imminent ({} bytes). Flushing before writing new packet of \
                 {encoded_packet_len} bytes",
                buffer.len()
            );

            if let Some(n) = self.flush_buffer(&mut buffer) {
                debug!("Flushed {n} bytes from buffer before appending new packet",);
            }
        }

        // Append the encoded packet into the buffer for later sending.
        buffer.extend_from_slice(&encoded_packet);

        trace!(
            "Appended packet of {encoded_packet_len} bytes to buffer (current buffer size: {})",
            buffer.len()
        );

        Ok(encoded_packet_len)
    }

    /// Sets the XTEA encryption key for outgoing packets.
    ///
    /// This key will be applied when flushing the buffer.
    pub fn set_xtea_key(&mut self, key: XTEAKey) {
        self.xtea_key = Some(key);

        if let Err(err) = self.xtea_key_shared.send(Some(key)) {
            error!(
                "Failed to update XTEA key for client {}: {:?}",
                self.addr, err
            );
        } else {
            debug!("XTEA key updated successfully for client {}", self.addr);
        }
    }

    /// Sets the checksum mode for outgoing packets.
    ///
    /// The checksum will be calculated and prepended or appended based on this mode.
    pub fn set_checksum_mode(&mut self, mode: ChecksumMode) {
        self.checksum_mode = Some(mode);

        debug!("Checksum mode set to {mode} for client {}", self.addr);
    }

    /// Returns the remote address of the connection.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Flushes the internal buffer, wraps it in an `OutgoingPacket`, and sends it
    /// to the writer task.
    ///
    /// Returns the number of bytes flushed if successful, or `None` if the buffer was empty.
    pub fn flush(&self) -> Option<usize> {
        let mut buffer = self.buffer.lock().ok()?;

        self.flush_buffer(&mut buffer)
    }

    fn flush_buffer(&self, buffer: &mut BytesMut) -> Option<usize> {
        if buffer.is_empty() {
            // trace!("No data to flush for client {}", self.addr);
            return None;
        }

        // Split the buffer to take ownership of the data and freeze it for immutability.
        let bytes = buffer.split().freeze();
        let bytes_len = bytes.len();

        // Create a new packet wrapping the frozen bytes.
        let mut packet = OutgoingPacket::new(bytes);

        // Apply XTEA encryption keys if set, to be used during encryption.
        if let Some(xtea_key) = self.xtea_key {
            packet.xtea_key(xtea_key);
            debug!("Applied XTEA key for client {}", self.addr);
        }

        // Apply checksum mode if set; checksum will be calculated before sending.
        if let Some(checksum_mode) = self.checksum_mode {
            packet.checksum_mode(checksum_mode);

            debug!(
                "Applied checksum mode {checksum_mode} for client {}",
                self.addr
            );
        }

        // Attempt to send the packet through the outgoing channel.
        match self.sender.send(packet) {
            Ok(..) => {
                info!(
                    "Flushed {bytes_len} bytes from buffer and sent to writer task for client {}",
                    self.addr
                );

                // Reserve buffer space for future packets to avoid reallocations.
                buffer.reserve(self.packet_policy.outgoing.max_length);

                Some(bytes_len)
            }
            Err(err) => {
                error!(
                    "Failed to send packet to writer task for client {}: {:?}",
                    self.addr, err
                );
                None
            }
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // Attempt to flush any remaining data when the connection is dropped.
        if let Some(flushed_bytes) = self.flush() {
            info!(
                "[{}] Flushed {} bytes from connection buffer during drop.",
                self.addr, flushed_bytes
            );
        } else {
            debug!(
                "[{}] No data to flush during drop of the connection.",
                self.addr
            );
        }
    }
}
