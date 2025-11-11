use bevy::prelude::*;
use bytes::BytesMut;
use std::time::Instant;
use suon_protocol::packets::{PACKET_KIND_SIZE, client::PacketKind};

use crate::server::packet::{
    PACKET_CHECKSUM_SIZE, PACKET_HEADER_SIZE,
    incoming::{IncomingPacket, login::PacketReadError},
};

/// Buffer responsible for accumulating and parsing login packets from a stream.
///
/// This structure manages an internal [`BytesMut`] buffer that stores
/// incoming raw bytes, including a 2-byte length prefix. It is designed to
/// handle partial reads from network streams, reconstruct complete packets,
/// and validate them according to the login protocol.
pub struct PacketBuffer {
    /// Internal buffer storing packet data, including the 2-byte length prefix.
    inner: BytesMut,
}

impl PacketBuffer {
    /// Creates a new [`PacketBuffer`] with a pre-allocated and zero-filled capacity.
    ///
    /// The total allocated space equals the provided `capacity`, including the
    /// length prefix area.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut inner = BytesMut::with_capacity(capacity);
        inner.resize(capacity, 0);

        Self { inner }
    }

    /// Attempts to extract a complete and validated login packet from the buffer.
    pub fn take_packet(&mut self, max_length: usize) -> Result<IncomingPacket, PacketReadError> {
        let buffer_length = self.inner.len();

        trace!("Checking for a complete packet in buffer ({buffer_length} bytes)");

        // Ensure the buffer has enough bytes for the length prefix
        if buffer_length < PACKET_HEADER_SIZE {
            trace!("Not enough bytes for length prefix");
            return Err(PacketReadError::IncompletePrefix {
                available: buffer_length,
                required: PACKET_HEADER_SIZE,
            });
        }

        // Read declared body length
        let declared_body_len = u16::from_le_bytes([self.inner[0], self.inner[1]]) as usize;
        if declared_body_len == 0 {
            warn!("Packet length is zero");
            return Err(PacketReadError::EmptyLength);
        }

        // Validate total packet length against maximum allowed
        let total_len = PACKET_HEADER_SIZE + declared_body_len;
        if total_len > max_length {
            warn!("Packet length {total_len} exceeds max allowed {max_length}");

            return Err(PacketReadError::LengthOutOfBounds {
                declared: total_len,
                max: max_length,
            });
        }

        // Ensure the buffer contains a full packet
        if buffer_length < total_len {
            trace!("Buffer incomplete ({total_len} bytes needed, {buffer_length} available)");

            return Err(PacketReadError::IncompletePacket {
                available: buffer_length,
                required: total_len,
            });
        }

        // Split out the complete packet and extract its body
        let packet_bytes = self.inner.split_to(total_len).freeze();
        let body_bytes = packet_bytes.slice(PACKET_HEADER_SIZE..);

        // Validate body length before checksum
        let min_body_len = PACKET_CHECKSUM_SIZE + PACKET_KIND_SIZE;
        if body_bytes.len() < min_body_len {
            warn!(
                "Packet body too short: {} bytes (minimum {min_body_len})",
                body_bytes.len()
            );

            return Err(PacketReadError::TooShort {
                actual: body_bytes.len(),
                min: min_body_len,
            });
        }

        // Extract and verify checksum
        let expected_checksum = suon_checksum::Adler32Checksum::from(u32::from_le_bytes(
            body_bytes[0..PACKET_CHECKSUM_SIZE].try_into().unwrap(),
        ));

        let payload_slice = &body_bytes[min_body_len..];
        if *expected_checksum > 0 {
            let actual_checksum = suon_checksum::Adler32Checksum::from(payload_slice);
            if expected_checksum != actual_checksum {
                warn!("Checksum mismatch: expected {expected_checksum}, actual {actual_checksum}");

                return Err(PacketReadError::ChecksumMismatch {
                    expected: *expected_checksum,
                    actual: *actual_checksum,
                });
            }
        }

        // Extract and parse packet kind
        let raw_kind = body_bytes[PACKET_CHECKSUM_SIZE];
        let packet_kind =
            PacketKind::try_from(raw_kind).map_err(|_| PacketReadError::UnknownId(raw_kind))?;
        if packet_kind != PacketKind::Login {
            warn!("Received non-login packet: kind {raw_kind}");
            return Err(PacketReadError::UnknownId(raw_kind));
        }

        let payload = body_bytes.slice(min_body_len..);

        trace!(
            "Successfully parsed login packet ({} bytes payload)",
            payload.len()
        );

        Ok(IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: packet_kind,
            buffer: payload,
        })
    }

    /// Returns a mutable reference to the payload section of the buffer.
    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Truncates the internal buffer to the specified length.
    pub fn truncate(&mut self, n: usize) {
        self.inner.truncate(n);
    }

    /// Returns the total number of bytes currently stored in the buffer.
    pub fn payload_len(&self) -> usize {
        self.inner.len()
    }
}
