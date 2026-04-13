use bevy::prelude::*;
use std::time::Instant;
use suon_protocol::packets::client::{Decodable, DecodableError};
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
    /// Failed to decode the packet from its buffer.
    #[error("failed to decode packet: {0}")]
    Decodable(#[from] DecodableError),

    /// The buffer contained extra bytes after decoding the packet.
    #[error("extra bytes remaining after decoding: {0}")]
    ExtraBytes(usize),
}

/// A strongly-typed packet event targeted at the originating client entity.
#[derive(Debug, EntityEvent)]
pub struct Packet<P: Decodable + Send + Sync + 'static> {
    /// The entity that sent the packet.
    #[event_target]
    pub(crate) entity: Entity,

    /// Timestamp when the packet was received.
    pub(crate) timestamp: Instant,

    /// The packet checksum for validation.
    pub(crate) checksum: Option<suon_checksum::Adler32Checksum>,

    /// Decoded packet payload.
    pub(crate) packet: P,
}

impl<P: Decodable + Send + Sync + 'static> Packet<P> {
    /// Returns the entity that originated the packet.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the timestamp when the packet was received.
    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    /// Returns the packet checksum, when present.
    pub fn checksum(&self) -> Option<suon_checksum::Adler32Checksum> {
        self.checksum
    }

    /// Returns the decoded packet payload.
    pub fn packet(&self) -> &P {
        &self.packet
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use suon_protocol::packets::client::PacketKind;

    #[derive(Debug, PartialEq, Eq)]
    struct Dummy;

    impl Decodable for Dummy {
        fn decode(_: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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

    fn build_packet<P: Decodable + Send + Sync + 'static>(
        buffer: &[u8],
    ) -> Result<Packet<P>, DecodeError> {
        let timestamp = Instant::now();
        let checksum = None;
        let entity = Entity::from_bits(7);
        let raw = Bytes::copy_from_slice(buffer);
        let mut bytes = raw.as_ref();
        let packet = P::decode(PacketKind::PingLatency, &mut bytes).map_err(DecodeError::from)?;
        if !bytes.is_empty() {
            return Err(DecodeError::ExtraBytes(bytes.len()));
        }

        Ok(Packet {
            entity,
            timestamp,
            checksum,
            packet,
        })
    }

    #[test]
    fn should_surface_decoder_failures() {
        let error =
            build_packet::<Dummy>(&[]).expect_err("Decoder errors should be surfaced to callers");

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
        let error = build_packet::<Dummy>(&[1, 2])
            .expect_err("Packets should reject decoders that leave unread bytes behind");

        assert!(matches!(error, DecodeError::ExtraBytes(1)));
    }

    #[test]
    fn should_decode_packets_when_kind_and_payload_match() {
        let decoded =
            build_packet::<Dummy>(&[1]).expect("Matching packets should decode successfully");

        assert_eq!(*decoded.packet(), Dummy);
    }

    #[test]
    fn should_expose_packet_metadata_through_getters() {
        let timestamp = Instant::now();
        let checksum = suon_checksum::Adler32Checksum::from(0xABCD1234);
        let packet = Packet {
            entity: Entity::from_bits(42),
            timestamp,
            checksum: Some(checksum),
            packet: Dummy,
        };

        assert_eq!(
            packet.entity(),
            Entity::from_bits(42),
            "entity should expose the entity that produced the packet"
        );

        assert_eq!(
            packet.timestamp(),
            timestamp,
            "timestamp should expose the reception instant stored in the packet"
        );

        assert_eq!(
            packet.checksum(),
            Some(checksum),
            "checksum should return the stored packet checksum when one is present"
        );
    }

    #[test]
    fn should_decode_typed_packets_with_metadata() {
        let timestamp = Instant::now();
        let checksum = suon_checksum::Adler32Checksum::from(0xABCD1234);
        let event = Packet {
            entity: Entity::from_bits(42),
            timestamp,
            checksum: Some(checksum),
            packet: Dummy,
        };

        assert_eq!(
            event.entity(),
            Entity::from_bits(42),
            "typed packet events should preserve the originating entity"
        );

        assert_eq!(
            event.timestamp(),
            timestamp,
            "typed packet events should preserve the original timestamp"
        );

        assert_eq!(
            event.checksum(),
            Some(checksum),
            "typed packet events should preserve the checksum metadata"
        );

        assert_eq!(
            event.event_target(),
            Entity::from_bits(42),
            "the event target should match the originating client entity"
        );
    }
}
