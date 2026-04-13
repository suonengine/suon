//! Client sell-npc-shop packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to sell an item stack to an NPC trade endpoint.
///
/// The payload identifies the item type, how many units from the selected
/// stack are being referenced, how many trade iterations should be applied, and
/// whether equipped items may be consumed as sale sources.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SellNpcShop {
    /// NPC-shop item type that should be sold.
    pub item_id: u16,

    /// Count or subtype byte taken from the referenced item stack.
    pub count: u8,

    /// Number of sale iterations requested for the selected offer.
    pub amount: u16,

    /// Whether equipped items may be consumed as sale sources.
    pub ignore_equipped: bool,
}

impl Decodable for SellNpcShop {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
        let packet = SellNpcShop::decode(PacketKind::SellNpcShop, &mut payload).unwrap();
        assert!(packet.ignore_equipped);
    }
}
