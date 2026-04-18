//! Client teleport packet.

use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

use super::prelude::*;

/// Packet sent by the client to request a teleport to a specific position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Teleport {
    /// Destination position requested by the client.
    pub position: Position,

    /// Destination floor requested by the client.
    pub floor: Floor,
}

impl Decodable for Teleport {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_teleport() {
        let mut payload: &[u8] = &[0x34, 0x12, 0x78, 0x56, 0x01];

        let packet = Teleport::decode(PacketKind::Teleport, &mut payload)
            .expect("Teleport packets should decode positions");

        assert_eq!(
            packet.position,
            Position {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert_eq!(packet.floor, Floor::from(0x01));
        assert!(payload.is_empty());
    }
}
