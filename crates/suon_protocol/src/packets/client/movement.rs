//! Client movement packet family.

use suon_position::direction::Direction;

use super::prelude::*;

/// Packet sent by the client to request a one-tile movement.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::MovePacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = MovePacket::decode_with_kind(PacketKind::MoveNorthEast, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::NorthEast);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovePacket {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl MovePacket {
    fn direction_from_kind(kind: PacketKind) -> Option<Direction> {
        match kind {
            PacketKind::MoveNorth => Some(Direction::North),
            PacketKind::MoveEast => Some(Direction::East),
            PacketKind::MoveSouth => Some(Direction::South),
            PacketKind::MoveWest => Some(Direction::West),
            PacketKind::MoveNorthEast => Some(Direction::NorthEast),
            PacketKind::MoveSouthEast => Some(Direction::SouthEast),
            PacketKind::MoveSouthWest => Some(Direction::SouthWest),
            PacketKind::MoveNorthWest => Some(Direction::NorthWest),
            _ => None,
        }
    }
}

impl Decodable for MovePacket {
    const KIND: PacketKind = PacketKind::MoveNorth;

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
    fn should_decode_move_packet_from_wire_kind() {
        let mut payload: &[u8] = &[];

        let packet = MovePacket::decode_with_kind(PacketKind::MoveNorthEast, &mut payload)
            .expect("Move packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::NorthEast);
        assert!(
            payload.is_empty(),
            "Move packet decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_accept_all_move_packet_kinds() {
        assert!(MovePacket::accepts_kind(PacketKind::MoveNorth));
        assert!(MovePacket::accepts_kind(PacketKind::MoveNorthWest));
        assert!(!MovePacket::accepts_kind(PacketKind::TurnNorth));
    }
}
