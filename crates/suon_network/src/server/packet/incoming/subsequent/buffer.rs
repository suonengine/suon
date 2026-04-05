//! Subsequent packet buffer parsing and XTEA-aware validation.

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
        let min_decrypted_len = PACKET_HEADER_SIZE + PACKET_KIND_SIZE;
        if decrypted_bytes.len() < min_decrypted_len {
            warn!(
                "Decrypted packet body too short: {} bytes (minimum required: {min_decrypted_len})",
                decrypted_bytes.len(),
            );

            return Err(PacketReadError::TooShort {
                actual: decrypted_bytes.len(),
                min: min_decrypted_len,
            });
        }

        // Drop the inner payload-length prefix added before encryption.
        let _ = decrypted_bytes.split_to(PACKET_HEADER_SIZE);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::packet::PACKET_HEADER_SIZE;
    use bytes::Bytes;

    const MAX_LENGTH: usize = 64;
    const XTEA_KEY: suon_xtea::XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];

    fn write_buffer(buffer: &mut PacketBuffer, bytes: &[u8]) {
        buffer.payload_mut()[..bytes.len()].copy_from_slice(bytes);
        buffer.truncate(bytes.len());
    }

    fn build_subsequent_packet_payload(payload: &[u8], checksum: u32) -> Vec<u8> {
        let mut plaintext = Vec::with_capacity(PACKET_HEADER_SIZE + payload.len());
        plaintext.extend_from_slice(&(payload.len() as u16).to_le_bytes());
        plaintext.extend_from_slice(payload);

        let encrypted = suon_xtea::encrypt(&plaintext, &XTEA_KEY);
        let mut bytes =
            Vec::with_capacity(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + encrypted.len());
        bytes.extend_from_slice(&((PACKET_CHECKSUM_SIZE + encrypted.len()) as u16).to_le_bytes());
        bytes.extend_from_slice(&checksum.to_le_bytes());
        bytes.extend_from_slice(&encrypted);
        bytes
    }

    #[test]
    fn should_reject_incomplete_prefix() {
        let mut buffer = PacketBuffer::with_capacity(8);
        buffer.truncate(1);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
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
        write_buffer(&mut buffer, &0_u16.to_le_bytes());

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Zero-length subsequent packets should be rejected");

        assert!(matches!(error, PacketReadError::EmptyLength));
    }

    #[test]
    fn should_reject_lengths_above_the_allowed_maximum() {
        let payload = [PacketKind::KeepAlive as u8];
        let packet_bytes = build_subsequent_packet_payload(&payload, 0);
        let mut buffer = PacketBuffer::with_capacity(32);
        write_buffer(&mut buffer, &packet_bytes);

        let error = buffer
            .take_packet(XTEA_KEY, packet_bytes.len() - 1)
            .expect_err("Packets above the configured maximum length should be rejected");

        assert!(matches!(
            error,
            PacketReadError::LengthOutOfBounds {
                declared,
                max
            } if declared == packet_bytes.len() && max == packet_bytes.len() - 1
        ));
    }

    #[test]
    fn should_reject_incomplete_subsequent_packets() {
        let payload = [PacketKind::KeepAlive as u8];
        let packet_bytes = build_subsequent_packet_payload(&payload, 0);
        let truncated_length = packet_bytes.len() - 1;
        let mut buffer = PacketBuffer::with_capacity(32);
        write_buffer(&mut buffer, &packet_bytes[..truncated_length]);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Missing encrypted bytes should keep the packet incomplete");

        assert!(matches!(
            error,
            PacketReadError::IncompletePacket {
                available,
                required
            } if available == truncated_length && required == packet_bytes.len()
        ));
    }

    #[test]
    fn should_reject_packets_shorter_than_the_checksum_field() {
        let mut buffer = PacketBuffer::with_capacity(8);
        let bytes = [3, 0, 0, 0, 0];
        write_buffer(&mut buffer, &bytes);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Subsequent packets must at least contain the checksum field");

        assert!(matches!(
            error,
            PacketReadError::TooShort {
                actual: 3,
                min: PACKET_CHECKSUM_SIZE
            }
        ));
    }

    #[test]
    fn should_decode_subsequent_packet_and_preserve_checksum() {
        let payload = [PacketKind::KeepAlive as u8];
        let packet_bytes = build_subsequent_packet_payload(&payload, 0);
        let checksum = *suon_checksum::Adler32Checksum::from(
            &packet_bytes[PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE..],
        );
        let packet_bytes = build_subsequent_packet_payload(&payload, checksum);

        let mut buffer = PacketBuffer::with_capacity(64);
        write_buffer(&mut buffer, &packet_bytes);

        let packet = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect("A valid encrypted subsequent packet should decode successfully");

        assert_eq!(packet.kind, PacketKind::KeepAlive);

        assert_eq!(
            packet.checksum,
            Some(suon_checksum::Adler32Checksum::from(checksum)),
            "Subsequent packets should preserve the validated checksum"
        );

        assert_eq!(packet.buffer, Bytes::new());
    }

    #[test]
    fn should_decode_subsequent_packet_without_preserving_zero_checksums() {
        const PAYLOAD: [u8; 3] = [PacketKind::PingLatency as u8, 1, 2];

        let packet_bytes = build_subsequent_packet_payload(&PAYLOAD, 0);
        let mut buffer = PacketBuffer::with_capacity(64);
        write_buffer(&mut buffer, &packet_bytes);

        let packet = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect("Zero checksums should skip validation and not be preserved");

        assert_eq!(packet.kind, PacketKind::PingLatency);
        assert_eq!(packet.checksum, None);
        assert_eq!(packet.buffer, Bytes::from_static(&[1, 2]));
    }

    #[test]
    fn should_reject_subsequent_packet_with_checksum_mismatch() {
        let payload = [PacketKind::KeepAlive as u8];
        let packet_bytes = build_subsequent_packet_payload(&payload, 0xDEADBEEF);

        let mut buffer = PacketBuffer::with_capacity(64);
        write_buffer(&mut buffer, &packet_bytes);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Checksum mismatches should reject subsequent packets");

        assert!(matches!(
            error,
            PacketReadError::ChecksumMismatch {
                expected: 0xDEADBEEF,
                ..
            }
        ));
    }

    #[test]
    fn should_surface_xtea_decryption_errors() {
        let mut buffer = PacketBuffer::with_capacity(32);
        let bytes = [9, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5];
        write_buffer(&mut buffer, &bytes);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Invalid encrypted payloads should surface a decryption error");

        assert!(matches!(error, PacketReadError::XteaDecryption(_)));
    }

    #[test]
    fn should_reject_subsequent_packet_with_unknown_kind_after_decryption() {
        let payload = [0xFF];
        let packet_bytes = build_subsequent_packet_payload(&payload, 0);
        let checksum = *suon_checksum::Adler32Checksum::from(
            &packet_bytes[PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE..],
        );
        let packet_bytes = build_subsequent_packet_payload(&payload, checksum);

        let mut buffer = PacketBuffer::with_capacity(64);
        write_buffer(&mut buffer, &packet_bytes);

        let error = buffer
            .take_packet(XTEA_KEY, MAX_LENGTH)
            .expect_err("Unknown packet ids should be rejected after decryption");

        assert!(matches!(error, PacketReadError::UnknownId(0xFF)));
    }

    #[test]
    fn should_report_subsequent_buffer_length_after_truncation() {
        let mut buffer = PacketBuffer::with_capacity(16);
        buffer.payload_mut().extend_from_slice(b"hello");
        buffer.truncate(5);

        assert_eq!(
            buffer.payload_len(),
            5,
            "payload_len should expose the number of bytes currently stored in the subsequent \
             buffer"
        );
    }
}
