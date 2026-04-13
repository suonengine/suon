//! Client face packet family.

use suon_position::direction::Direction;

use super::prelude::*;

/// Packet sent by the client to request a facing change.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::Face};
///
/// let mut payload: &[u8] = &[];
/// let packet = Face::decode(PacketKind::FaceWest, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::West);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl Face {
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

impl Decodable for Face {
    fn decode(kind: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
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

        let packet = Face::decode(PacketKind::FaceWest, &mut payload)
            .expect("Face packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::West);
        assert!(
            payload.is_empty(),
            "Face packet decoding should not consume any payload bytes"
        );
    }
}
