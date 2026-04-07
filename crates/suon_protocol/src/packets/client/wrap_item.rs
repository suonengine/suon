//! Client wrap-item packet.

use suon_position::{floor::Floor, position::Position};

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapItemPacket {
    pub position: Position,
    pub floor: Floor,
    pub item_id: u16,
    pub stack_position: u8,
}

impl Decodable for WrapItemPacket {
    const KIND: PacketKind = PacketKind::WrapItem;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
    fn should_decode_wrap_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];

        assert_eq!(
            WrapItemPacket::decode(&mut payload).unwrap().item_id,
            0x1234
        );
    }
}
