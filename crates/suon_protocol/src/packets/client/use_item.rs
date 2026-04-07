//! Client use-item packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseItemPacket {
    pub position: Position,
    pub floor: Floor,
    pub item_id: u16,
    pub stack_position: u8,
    pub use_index: u8,
}

impl Decodable for UseItemPacket {
    const KIND: PacketKind = PacketKind::UseItem;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            use_index: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_use_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 4];
        let packet = UseItemPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.use_index, 4);
    }
}
