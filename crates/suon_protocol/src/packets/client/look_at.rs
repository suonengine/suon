//! Client look-at packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to request the description of an object or creature at a map position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookAt {
    /// Map coordinates of the tile being inspected.
    pub position: Position,

    /// Floor component of the inspected tile coordinates.
    pub floor: Floor,

    /// Client-sent thing type marker consumed by the server before the stack slot.
    pub item_id: u16,

    /// Stack slot of the thing to describe at the addressed tile.
    pub stack_position: u8,
}

impl Decodable for LookAt {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_look_at() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];
        let packet = LookAt::decode(PacketKind::LookAt, &mut payload).unwrap();

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.stack_position, 3);
        assert!(payload.is_empty());
    }
}
