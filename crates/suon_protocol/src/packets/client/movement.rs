//! Client step packet family.

use suon_position::prelude::*;

use super::prelude::*;

/// Packet sent by the client to request a one-tile step.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::StepPacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = StepPacket::decode_with_kind(PacketKind::StepNorthEast, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::NorthEast);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepPacket {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl StepPacket {
    fn direction_from_kind(kind: PacketKind) -> Option<Direction> {
        match kind {
            PacketKind::StepNorth => Some(Direction::North),
            PacketKind::StepEast => Some(Direction::East),
            PacketKind::StepSouth => Some(Direction::South),
            PacketKind::StepWest => Some(Direction::West),
            PacketKind::StepNorthEast => Some(Direction::NorthEast),
            PacketKind::StepSouthEast => Some(Direction::SouthEast),
            PacketKind::StepSouthWest => Some(Direction::SouthWest),
            PacketKind::StepNorthWest => Some(Direction::NorthWest),
            _ => None,
        }
    }
}

impl Decodable for StepPacket {
    const KIND: PacketKind = PacketKind::StepNorth;

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
    fn should_decode_step_packet_from_wire_kind() {
        let mut payload: &[u8] = &[];

        let packet = StepPacket::decode_with_kind(PacketKind::StepNorthEast, &mut payload)
            .expect("Step packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::NorthEast);
        assert!(
            payload.is_empty(),
            "Step packet decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_accept_all_step_packet_kinds() {
        assert!(StepPacket::accepts_kind(PacketKind::StepNorth));
        assert!(StepPacket::accepts_kind(PacketKind::StepNorthWest));
        assert!(!StepPacket::accepts_kind(PacketKind::FaceNorth));
    }
}
