use bevy::prelude::*;
use bytes::BytesMut;
use std::time::Instant;
use suon_protocol::packets::{PACKET_KIND_SIZE, client::PacketKind};

use crate::server::packet::{
    PACKET_CHECKSUM_SIZE, PACKET_HEADER_SIZE,
    incoming::{IncomingPacket, subsequent::PacketReadError},
};

/// Buffer responsible for accumulating and parsing subsequent packets from a stream.
///
/// This structure manages an internal [`BytesMut`] buffer that stores
/// incoming raw bytes, including a 2-byte length prefix. It is designed to
/// handle partial reads from network streams, reconstruct complete packets,
/// and validate them according to the subsequent protocol.
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

    /// Attempts to extract a complete and validated subsequent packet from the buffer.
    pub fn take_packet(
        &mut self,
        xtea_key: suon_xtea::XTEAKey,
        max_length: usize,
    ) -> Result<IncomingPacket, PacketReadError> {
        let buffer_length = self.inner.len();

        trace!("Checking for complete packet in buffer ({buffer_length} bytes)");

        // Ensure the buffer has enough bytes for the length prefix
        if buffer_length < PACKET_HEADER_SIZE {
            trace!(
                "Insufficient data for length prefix: {buffer_length} available, {} required",
                PACKET_HEADER_SIZE,
            );

            return Err(PacketReadError::IncompletePrefix {
                available: buffer_length,
                required: PACKET_HEADER_SIZE,
            });
        }

        // Read declared body length
        let declared_body_len = u16::from_le_bytes([self.inner[0], self.inner[1]]) as usize;
        if declared_body_len == 0 {
            warn!("Invalid packet: declared body length is zero");
            return Err(PacketReadError::EmptyLength);
        }

        // Validate total packet length against maximum allowed
        let total_len = PACKET_HEADER_SIZE + declared_body_len;
        if total_len > max_length {
            warn!("Packet length {total_len} exceeds maximum allowed {max_length}");

            return Err(PacketReadError::LengthOutOfBounds {
                declared: total_len,
                max: max_length,
            });
        }

        // Ensure the buffer contains a full packet
        if buffer_length < total_len {
            trace!("Incomplete packet: {buffer_length} bytes available, {total_len} required");

            return Err(PacketReadError::IncompletePacket {
                available: buffer_length,
                required: total_len,
            });
        }

        // Split out the complete packet and extract its body
        let packet_bytes = self.inner.split_to(total_len).freeze();
        let body_bytes = packet_bytes.slice(PACKET_HEADER_SIZE..);

        // Validate body length before checksum
        if body_bytes.len() < PACKET_CHECKSUM_SIZE {
            warn!(
                "Packet body too short: {} bytes (minimum required: {PACKET_CHECKSUM_SIZE})",
                body_bytes.len()
            );

            return Err(PacketReadError::TooShort {
                actual: body_bytes.len(),
                min: PACKET_CHECKSUM_SIZE,
            });
        }

        // Extract and verify checksum
        let expected_checksum = suon_checksum::Adler32Checksum::from(u32::from_le_bytes(
            body_bytes[0..PACKET_CHECKSUM_SIZE].try_into().unwrap(),
        ));

        let payload_slice = &body_bytes[PACKET_CHECKSUM_SIZE..];
        if *expected_checksum > 0 {
            let actual_checksum = suon_checksum::Adler32Checksum::from(payload_slice);
            if expected_checksum != actual_checksum {
                warn!("Checksum mismatch: expected {expected_checksum}, got {actual_checksum}");

                return Err(PacketReadError::ChecksumMismatch {
                    expected: *expected_checksum,
                    actual: *actual_checksum,
                });
            }
        }

        // Decrypt payload using XTEA
        let mut decrypted_bytes: BytesMut = suon_xtea::decrypt(payload_slice, &xtea_key)?.into();

        // Validate decrypted payload length
        if decrypted_bytes.len() < PACKET_KIND_SIZE {
            warn!(
                "Decrypted packet body too short: {} bytes (minimum required: {PACKET_KIND_SIZE})",
                decrypted_bytes.len()
            );

            return Err(PacketReadError::TooShort {
                actual: decrypted_bytes.len(),
                min: PACKET_KIND_SIZE,
            });
        }

        // Extract and parse packet kind
        let kind_bytes = decrypted_bytes.split_to(PACKET_KIND_SIZE);
        let packet_kind =
            PacketKind::try_from(u8::from_le_bytes(kind_bytes.as_ref().try_into().unwrap()))
                .map_err(PacketReadError::UnknownId)?;

        let payload = decrypted_bytes.freeze();

        trace!(
            "Successfully parsed subsequent packet ({} bytes payload)",
            payload.len()
        );

        Ok(IncomingPacket {
            timestamp: Instant::now(),
            checksum: if *expected_checksum > 0 {
                Some(expected_checksum)
            } else {
                None
            },
            kind: packet_kind,
            buffer: payload,
        })
    }

    /// Returns a mutable reference to the internal [`BytesMut`] buffer.
    pub fn payload_mut(&mut self) -> &mut BytesMut {
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
