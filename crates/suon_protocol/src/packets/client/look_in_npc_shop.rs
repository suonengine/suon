//! Client look-in-npc-shop packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookInNpcShop {
    pub item_id: u16,
    pub count: u8,
}

impl Decodable for LookInNpcShop {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            item_id: bytes.get_u16()?,
            count: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_look_in_npc_shop() {
        let mut payload: &[u8] = &[0x34, 0x12, 7];
        let packet = LookInNpcShop::decode(PacketKind::LookInNpcShop, &mut payload).unwrap();
        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.count, 7);
    }
}
