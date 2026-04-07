//! Client use-item-with-creature packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseItemWithCreaturePacket {
    pub position: Position,
    pub floor: Floor,
    pub item_id: u16,
    pub stack_position: u8,
    pub creature_id: u32,
}

impl Decodable for UseItemWithCreaturePacket {
    const KIND: PacketKind = PacketKind::UseItemWithCreature;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            creature_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_use_item_with_creature() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 0x78, 0x56, 0x34, 0x12];
        let packet = UseItemWithCreaturePacket::decode(&mut payload).unwrap();
        assert_eq!(packet.creature_id, 0x12345678);
    }
}
