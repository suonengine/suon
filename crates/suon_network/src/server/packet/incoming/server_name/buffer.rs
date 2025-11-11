use bevy::{
    log::{Level, tracing::enabled},
    prelude::*,
};
use bytes::BytesMut;
use std::time::Instant;
use suon_protocol::packets::client::PacketKind;

use crate::server::packet::{PACKET_HEADER_SIZE, incoming::IncomingPacket};

/// A buffer used to accumulate and finalize server name packets from a stream.
///
/// The `PacketBuffer` stores raw packet data, including a fixed-size prefix
/// reserved for the payload length. Incoming bytes are written into this buffer
/// until a full packet is detected (terminated by [`NEWLINE_TERMINATOR`]),
/// at which point the buffer is finalized and converted into an [`IncomingPacket`].
///
/// This structure automatically handles a special “empty packet” case and ensures
/// that the packet length prefix is always updated before producing the final packet.
pub struct PacketBuffer {
    /// Internal buffer storing the packet data, including the length prefix.
    inner: BytesMut,

    /// Tracks whether the special empty packet has already been handled.
    empty_packet_checked: bool,
}

impl PacketBuffer {
    /// Byte used to identify the end of a packet.
    pub const NEWLINE_TERMINATOR: u8 = b'\n';

    /// Creates a new buffer with the specified payload capacity.
    ///
    /// The total internal size will be `capacity + PREFIX_LENGTH`,
    /// and all bytes will be initialized to zero.
    pub fn with_capacity(capacity: usize) -> Self {
        let total = capacity + PACKET_HEADER_SIZE;
        trace!("Initializing PacketBuffer with total capacity {}", total);

        let mut inner = BytesMut::with_capacity(total);
        inner.resize(total, 0);

        info!(
            "PacketBuffer created with {} bytes total ({} prefix + {} payload)",
            PACKET_HEADER_SIZE,
            total - PACKET_HEADER_SIZE,
            PACKET_HEADER_SIZE,
        );

        Self {
            inner,
            empty_packet_checked: false,
        }
    }

    /// Attempts to extract a complete packet from the buffer.
    pub fn take_packet(&mut self) -> Option<IncomingPacket> {
        let buffer_length = self.inner.len();

        trace!(
            "Checking buffer for complete packet (payload_len = {}, total_len = {buffer_length})",
            self.payload_len()
        );

        // Handle the special "empty packet" case once.
        if !self.empty_packet_checked {
            self.empty_packet_checked = true;

            // If the second payload byte is 0, treat this as a special empty packet.
            if self.payload_len() > 1 && self.inner[PACKET_HEADER_SIZE] == 0 {
                info!("Detected special empty packet");
                return Some(self.build_packet());
            }
        }

        // Regular packet: must end with newline.
        let newline_byte = self.inner.last()?;
        if newline_byte != &Self::NEWLINE_TERMINATOR {
            trace!("Packet incomplete: last byte is not newline terminator");
            return None;
        }

        trace!("Newline terminator found, finalizing packet");

        // Remove the newline terminator.
        self.inner.truncate(buffer_length - 1);

        Some(self.build_packet())
    }

    /// Returns a mutable reference to the payload section of the buffer,
    /// excluding the reserved length prefix region.
    ///
    /// This allows writing data directly into the payload area without
    /// overwriting the prefix.
    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.inner[PACKET_HEADER_SIZE..]
    }

    /// Truncates the internal buffer to the specified length.
    ///
    /// The truncated length includes the reserved prefix region.
    /// If `n` is smaller than [`PREFIX_LENGTH`], the prefix is preserved.
    pub fn truncate(&mut self, n: usize) {
        let n = n.saturating_add(PACKET_HEADER_SIZE);
        trace!("Truncating buffer from {} to {} bytes", self.inner.len(), n);
        self.inner.truncate(n);
    }

    /// Returns the current payload length.
    #[inline]
    pub fn payload_len(&self) -> usize {
        self.inner.len().saturating_sub(PACKET_HEADER_SIZE)
    }

    /// Writes the current payload length into the reserved prefix region,
    /// then constructs and returns an [`IncomingPacket`].
    ///
    /// This method consumes the internal buffer, freezing it into an
    /// immutable byte sequence for transmission or further processing.
    fn build_packet(&mut self) -> IncomingPacket {
        let payload_length = self.payload_len() as u16;
        debug!("Building packet with payload length {}", payload_length);

        self.inner[..PACKET_HEADER_SIZE].copy_from_slice(&payload_length.to_le_bytes());
        trace!(
            "Length prefix written as {:?}",
            &self.inner[..PACKET_HEADER_SIZE]
        );

        if enabled!(Level::INFO) {
            let payload_bytes =
                &self.inner[PACKET_HEADER_SIZE..PACKET_HEADER_SIZE + payload_length as usize];
            let payload_utf8 = std::str::from_utf8(payload_bytes).unwrap_or("<invalid UTF-8>");
            info!("ServerName packet payload (UTF-8): {}", payload_utf8);
        }

        let frozen = self.inner.split().freeze();

        info!(
            "Finalized packet ({} bytes total, {} payload + {} prefix)",
            frozen.len(),
            payload_length,
            PACKET_HEADER_SIZE
        );

        IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::ServerName,
            buffer: frozen,
        }
    }
}
