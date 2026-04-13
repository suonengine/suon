//! Client throw-item packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThrowItem {
    pub from_position: Position,
    pub from_floor: Floor,
    pub item_id: u16,
    pub from_stack_position: u8,
    pub to_position: Position,
    pub to_floor: Floor,
    pub count: u8,
}

impl Decodable for ThrowItem {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            from_position: bytes.get_position()?,
            from_floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            from_stack_position: bytes.get_u8()?,
            to_position: bytes.get_position()?,
            to_floor: bytes.get_floor()?,
            count: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_throw_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 4, 0, 5, 0, 8, 9];
        let packet = ThrowItem::decode(PacketKind::ThrowItem, &mut payload).unwrap();
        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.count, 9);
    }
}
