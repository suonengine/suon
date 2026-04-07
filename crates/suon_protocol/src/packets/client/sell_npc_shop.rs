//! Client sell-npc-shop packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellNpcShopPacket {
    pub item_id: u16,
    pub count: u8,
    pub amount: u16,
    pub ignore_equipped: bool,
}

impl Decodable for SellNpcShopPacket {
    const KIND: PacketKind = PacketKind::SellNpcShop;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            item_id: bytes.get_u16()?,
            count: bytes.get_u8()?,
            amount: bytes.get_u16()?,
            ignore_equipped: bytes.get_bool()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_sell_npc_shop() {
        let mut payload: &[u8] = &[0x34, 0x12, 7, 0x02, 0x00, 1];
        let packet = SellNpcShopPacket::decode(&mut payload).unwrap();
        assert!(packet.ignore_equipped);
    }
}
