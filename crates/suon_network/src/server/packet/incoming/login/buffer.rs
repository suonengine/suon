//! Login packet buffer parsing and validation.

use bevy::prelude::*;
use bytes::BytesMut;
use std::time::Instant;
use suon_protocol::prelude::*;
use suon_protocol_client::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::packet::PACKET_HEADER_SIZE;
    use bytes::Bytes;

    const MAX_LENGTH: usize = 64;

    fn build_login_packet_payload(payload: &[u8], checksum: u32, kind: u8) -> Vec<u8> {
        let total_len =
            PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_KIND_SIZE + payload.len();
        let mut bytes = Vec::with_capacity(total_len);
        bytes.extend_from_slice(
            &((PACKET_CHECKSUM_SIZE + PACKET_KIND_SIZE + payload.len()) as u16).to_le_bytes(),
        );
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes.push(kind);
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn should_reject_incomplete_prefix() {
        let mut buffer = PacketBuffer::with_capacity(8);
        buffer.truncate(1);

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Short buffers should fail before reading the length prefix");

        assert!(matches!(
            error,
            PacketReadError::IncompletePrefix {
                available: 1,
                required: PACKET_HEADER_SIZE
            }
        ));
    }

    #[test]
    fn should_reject_empty_declared_length() {
        let mut buffer = PacketBuffer::with_capacity(8);
        buffer.payload_mut()[..PACKET_HEADER_SIZE].copy_from_slice(&0_u16.to_le_bytes());
        buffer.truncate(PACKET_HEADER_SIZE);

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Zero-length login packets should be rejected");

        assert!(matches!(error, PacketReadError::EmptyLength));
    }

    #[test]
    fn should_reject_lengths_above_the_allowed_maximum() {
        let mut buffer = PacketBuffer::with_capacity(16);
        let packet_bytes = build_login_packet_payload(b"x", 0, PacketKind::Login as u8);
        buffer.payload_mut()[..packet_bytes.len()].copy_from_slice(&packet_bytes);
        buffer.truncate(packet_bytes.len());

        let error = buffer
            .take_packet(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_KIND_SIZE)
            .expect_err("Packets above the configured maximum length should be rejected");

        assert!(matches!(
            error,
            PacketReadError::LengthOutOfBounds {
                declared: 8,
                max: 7
            }
        ));
    }

    #[test]
    fn should_reject_incomplete_login_packets() {
        let mut buffer = PacketBuffer::with_capacity(16);
        let packet_bytes = build_login_packet_payload(b"abc", 0, PacketKind::Login as u8);
        let truncated_length = packet_bytes.len() - 1;
        buffer.payload_mut()[..truncated_length].copy_from_slice(&packet_bytes[..truncated_length]);
        buffer.truncate(truncated_length);

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Missing bytes should keep the login packet incomplete");

        assert!(matches!(
            error,
            PacketReadError::IncompletePacket {
                available,
                required
            } if available == truncated_length && required == packet_bytes.len()
        ));
    }

    #[test]
    fn should_reject_login_packets_that_are_too_short_for_kind_and_checksum() {
        let mut buffer = PacketBuffer::with_capacity(16);
        let bytes = [4, 0, 0, 0, 0, 0];
        buffer.payload_mut()[..bytes.len()].copy_from_slice(&bytes);
        buffer.truncate(bytes.len());

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Login packets shorter than checksum plus kind should be rejected");

        assert!(matches!(
            error,
            PacketReadError::TooShort {
                actual: 4,
                min
            } if min == PACKET_CHECKSUM_SIZE + PACKET_KIND_SIZE
        ));
    }

    #[test]
    fn should_decode_login_packet_without_checksum_validation_when_checksum_is_zero() {
        const PAYLOAD: &[u8] = b"login";

        let mut buffer = PacketBuffer::with_capacity(32);
        let packet_bytes = build_login_packet_payload(PAYLOAD, 0, PacketKind::Login as u8);
        buffer.payload_mut()[..packet_bytes.len()].copy_from_slice(&packet_bytes);
        buffer.truncate(packet_bytes.len());

        let packet = buffer
            .take_packet(MAX_LENGTH)
            .expect("A valid login packet should decode successfully");

        assert_eq!(packet.kind, PacketKind::Login);
        assert_eq!(packet.checksum, None);
        assert_eq!(packet.buffer, Bytes::copy_from_slice(PAYLOAD));
    }

    #[test]
    fn should_validate_login_packet_checksums_against_the_payload() {
        const PAYLOAD: &[u8] = b"login";

        let mut buffer = PacketBuffer::with_capacity(32);
        let checksum = *suon_checksum::Adler32Checksum::from(PAYLOAD);
        let packet_bytes = build_login_packet_payload(PAYLOAD, checksum, PacketKind::Login as u8);
        buffer.payload_mut()[..packet_bytes.len()].copy_from_slice(&packet_bytes);
        buffer.truncate(packet_bytes.len());

        let packet = buffer
            .take_packet(MAX_LENGTH)
            .expect("Matching payload checksums should allow login packets");

        assert_eq!(packet.kind, PacketKind::Login);
        assert_eq!(packet.checksum, None);
        assert_eq!(packet.buffer, Bytes::copy_from_slice(PAYLOAD));
    }

    #[test]
    fn should_reject_login_packets_with_non_login_kinds() {
        let mut buffer = PacketBuffer::with_capacity(32);
        let packet_bytes = build_login_packet_payload(b"login", 0, PacketKind::KeepAlive as u8);
        buffer.payload_mut()[..packet_bytes.len()].copy_from_slice(&packet_bytes);
        buffer.truncate(packet_bytes.len());

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Only login packets should be accepted during the login stage");

        assert!(matches!(
            error,
            PacketReadError::UnknownId(kind) if kind == PacketKind::KeepAlive as u8
        ));
    }

    #[test]
    fn should_reject_login_packet_with_checksum_mismatch() {
        let mut buffer = PacketBuffer::with_capacity(32);
        let packet_bytes =
            build_login_packet_payload(b"login", 0xDEADBEEF, PacketKind::Login as u8);
        buffer.payload_mut()[..packet_bytes.len()].copy_from_slice(&packet_bytes);
        buffer.truncate(packet_bytes.len());

        let error = buffer
            .take_packet(MAX_LENGTH)
            .expect_err("Invalid checksums should reject login packets");

        assert!(matches!(
            error,
            PacketReadError::ChecksumMismatch {
                expected: 0xDEADBEEF,
                ..
            }
        ));
    }

    #[test]
    fn should_report_login_buffer_length_after_truncation() {
        let mut buffer = PacketBuffer::with_capacity(16);
        buffer.payload_mut()[..5].copy_from_slice(b"hello");
        buffer.truncate(5);

        assert_eq!(
            buffer.payload_len(),
            5,
            "payload_len should expose the number of bytes currently stored in the login buffer"
        );
    }
}
