//! Client step packet family.

use suon_position::direction::Direction;

use super::prelude::*;

/// Packet sent by the client to request a one-tile step.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::Step};
///
/// let mut payload: &[u8] = &[];
/// let packet = Step::decode(PacketKind::StepNorthEast, &mut payload).unwrap();
///
/// assert_eq!(packet.direction, Direction::NorthEast);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    /// Direction requested by the client.
    pub direction: Direction,
}

impl Step {
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

impl Decodable for Step {
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
    fn should_decode_step_packet_from_wire_kind() {
        let mut payload: &[u8] = &[];

        let packet = Step::decode(PacketKind::StepNorthEast, &mut payload)
            .expect("Step packets should decode direction from the packet kind");

        assert_eq!(packet.direction, Direction::NorthEast);
        assert!(
            payload.is_empty(),
            "Step packet decoding should not consume any payload bytes"
        );
    }
}
