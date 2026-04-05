use bevy::prelude::*;
use bytes::Bytes;
use std::time::Instant;
use suon_protocol::packets::client::{Decodable, DecodableError, PacketKind};
use thiserror::Error;

pub mod incoming;
pub mod outgoing;

/// Number of bytes used for the packet checksum field.
/// This checksum is used to verify packet integrity after transmission.
pub(crate) const PACKET_CHECKSUM_SIZE: usize = 4;

/// Length of the payload header, in bytes.
/// This header precedes the actual packet body and may be used in codec routines.
pub(crate) const PACKET_HEADER_SIZE: usize = 2;

/// Errors that can occur while decoding a `Packet`.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// The packet KIND does not match the expected type.
    #[error("packet KIND mismatch: expected {expected}, found {found}")]
    KindMismatch {
        expected: PacketKind,
        found: PacketKind,
    },

    /// Failed to decode the packet from its buffer.
    #[error("failed to decode packet: {0}")]
    Decodable(#[from] DecodableError),

    /// The buffer contained extra bytes after decoding the packet.
    #[error("extra bytes remaining after decoding: {0}")]
    ExtraBytes(usize),
}

/// Represents a decoded packet message from a client entity.
#[derive(Message)]
pub struct Packet {
    /// The client entity that sent the packet.
    pub(crate) client: Entity,

    /// Timestamp when the packet was received.
    pub(crate) timestamp: Instant,

    /// The packet checksum for validation.
    pub(crate) checksum: Option<suon_checksum::Adler32Checksum>,

    /// The packet kind identifier.
    pub(crate) kind: PacketKind,

    /// Raw packet bytes.
    pub(crate) buffer: Bytes,
}

impl Packet {
    /// Returns the client entity associated with this packet.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the timestamp when the packet was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns the checksum of the packet.
    pub fn checksum(&self) -> Option<suon_checksum::Adler32Checksum> {
        self.checksum
    }

    /// Attempts to decode the raw buffer into a strongly-typed packet.
    ///
    /// ### Steps
    /// 1. Verify that the packet KIND matches the expected type `P`.
    /// 2. Call `P::decode` on the buffer to attempt decoding.
    /// 3. Return an error if decoding fails or if extra bytes remain.
    ///
    /// ### Returns
    /// `Ok(P)` if decoding succeeds, otherwise `Err(PacketDecodeError)`.
    pub fn decode<P: Decodable>(&self) -> Result<P, DecodeError> {
        // Ensure packet KIND matches
        if self.kind != P::KIND {
            warn!(
                "Packet kind mismatch for client {}: expected {:?}, found {:?}",
                self.client,
                P::KIND,
                self.kind
            );

            return Err(DecodeError::KindMismatch {
                expected: P::KIND,
                found: self.kind,
            });
        }

        // Decode the packet from the buffer
        let mut bytes = &self.buffer[..];

        let packet = match P::decode(&mut bytes) {
            Ok(p) => p,
            Err(err) => {
                error!(
                    "Failed to decode packet for client {}: {:?}",
                    self.client, err
                );
                return Err(err.into());
            }
        };

        // Check for leftover bytes
        if !bytes.is_empty() {
            warn!(
                "Extra bytes detected after decoding packet for client {}: {} bytes",
                self.client,
                bytes.len()
            );

            return Err(DecodeError::ExtraBytes(bytes.len()));
        }

        debug!("Successfully decoded packet for client {}", self.client);

        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct PingLatencyPacket;

    impl Decodable for PingLatencyPacket {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
            if bytes.is_empty() {
                return Err(DecodableError::Decoder(
                    suon_protocol::packets::decoder::DecoderError::Incomplete {
                        expected: 1,
                        available: 0,
                    },
                ));
            }

            *bytes = &bytes[1..];

            Ok(Self)
        }
    }

    fn build_packet(kind: PacketKind, buffer: &[u8]) -> Packet {
        Packet {
            client: Entity::from_bits(7),
            timestamp: Instant::now(),
            checksum: None,
            kind,
            buffer: Bytes::copy_from_slice(buffer),
        }
    }

    #[test]
    fn should_reject_packets_with_mismatched_kinds() {
        let packet = build_packet(PacketKind::KeepAlive, &[1]);

        let error = packet
            .decode::<PingLatencyPacket>()
            .expect_err("Packets should reject decoders for a different packet kind");

        assert!(matches!(
            error,
            DecodeError::KindMismatch {
                expected: PacketKind::PingLatency,
                found: PacketKind::KeepAlive
            }
        ));
    }

    #[test]
    fn should_surface_decoder_failures() {
        let packet = build_packet(PacketKind::PingLatency, &[]);

        let error = packet
            .decode::<PingLatencyPacket>()
            .expect_err("Decoder errors should be surfaced to callers");

        assert!(matches!(
            error,
            DecodeError::Decodable(DecodableError::Decoder(
                suon_protocol::packets::decoder::DecoderError::Incomplete {
                    expected: 1,
                    available: 0
                }
            ))
        ));
    }

    #[test]
    fn should_reject_packets_with_extra_bytes_after_decoding() {
        let packet = build_packet(PacketKind::PingLatency, &[1, 2]);

        let error = packet
            .decode::<PingLatencyPacket>()
            .expect_err("Packets should reject decoders that leave unread bytes behind");

        assert!(matches!(error, DecodeError::ExtraBytes(1)));
    }

    #[test]
    fn should_decode_packets_when_kind_and_payload_match() {
        let packet = build_packet(PacketKind::PingLatency, &[1]);
        let decoded = packet
            .decode::<PingLatencyPacket>()
            .expect("Matching packets should decode successfully");

        assert_eq!(decoded, PingLatencyPacket);
    }
}
