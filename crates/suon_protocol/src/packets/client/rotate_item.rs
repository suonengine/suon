//! Client rotate-item packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to rotate an item at a specific map position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RotateItem {
    /// Map coordinates of the tile containing the item to rotate.
    pub position: Position,
    /// Floor component of the addressed tile coordinates.
    pub floor: Floor,
    /// Advertised item type currently present at the addressed slot.
    pub item_id: u16,
    /// Stack slot of the item that should be rotated.
    pub stack_position: u8,
}

impl Decodable for RotateItem {
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
    fn should_decode_rotate_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];
        let packet = RotateItem::decode(PacketKind::RotateItem, &mut payload).unwrap();
        assert_eq!(packet.stack_position, 3);
    }
}
