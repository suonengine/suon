//! Client face packet family.

use suon_position::prelude::*;

use super::prelude::*;

/// Packet sent by the client to request a facing change.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::FacePacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = FacePacket::decode_with_kind(PacketKind::FaceWest, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::West);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacePacket {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl FacePacket {
    fn direction_from_kind(kind: PacketKind) -> Option<Direction> {
        match kind {
            PacketKind::FaceNorth => Some(Direction::North),
            PacketKind::FaceEast => Some(Direction::East),
            PacketKind::FaceSouth => Some(Direction::South),
            PacketKind::FaceWest => Some(Direction::West),
            _ => None,
        }
    }
}

impl Decodable for FacePacket {
    const KIND: PacketKind = PacketKind::FaceNorth;

    fn accepts_kind(kind: PacketKind) -> bool {
        Self::direction_from_kind(kind).is_some()
    }

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Err(DecodableError::InvalidFieldValue {
            field: "packet_kind",
            value: Self::KIND as u8,
        })
    }

    fn decode_with_kind(kind: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        let Some(direction) = Self::direction_from_kind(kind) else {
            return Err(DecodableError::InvalidFieldValue {
                field: "packet_kind",
                value: kind as u8,
            });
        };

        Ok(Self { direction })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_face_packet_from_wire_kind() {
        let mut payload: &[u8] = &[];

        let packet = FacePacket::decode_with_kind(PacketKind::FaceWest, &mut payload)
            .expect("Face packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::West);
        assert!(
            payload.is_empty(),
            "Face packet decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_accept_only_face_packet_kinds() {
        assert!(FacePacket::accepts_kind(PacketKind::FaceNorth));
        assert!(!FacePacket::accepts_kind(PacketKind::StepNorth));
    }
}
