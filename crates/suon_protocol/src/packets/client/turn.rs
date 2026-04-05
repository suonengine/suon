//! Client turn packet family.

use suon_position::direction::Direction;

use super::prelude::*;

/// Packet sent by the client to request a facing change.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::TurnPacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = TurnPacket::decode_with_kind(PacketKind::TurnWest, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::West);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnPacket {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl TurnPacket {
    fn direction_from_kind(kind: PacketKind) -> Option<Direction> {
        match kind {
            PacketKind::TurnNorth => Some(Direction::North),
            PacketKind::TurnEast => Some(Direction::East),
            PacketKind::TurnSouth => Some(Direction::South),
            PacketKind::TurnWest => Some(Direction::West),
            _ => None,
        }
    }
}

impl Decodable for TurnPacket {
    const KIND: PacketKind = PacketKind::TurnNorth;

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
    fn should_decode_turn_packet_from_wire_kind() {
        let mut payload: &[u8] = &[];

        let packet = TurnPacket::decode_with_kind(PacketKind::TurnWest, &mut payload)
            .expect("Turn packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::West);
        assert!(
            payload.is_empty(),
            "Turn packet decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_accept_only_turn_packet_kinds() {
        assert!(TurnPacket::accepts_kind(PacketKind::TurnNorth));
        assert!(!TurnPacket::accepts_kind(PacketKind::MoveNorth));
    }
}
