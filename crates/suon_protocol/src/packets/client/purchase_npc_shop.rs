//! Client purchase-npc-shop packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to buy an item from an NPC shop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurchaseNpcShop {
    /// NPC-shop item type being purchased.
    pub item_id: u16,
    /// Count or subtype byte attached to each purchased item entry.
    pub count: u8,
    /// Number of trade iterations requested for the selected offer.
    pub amount: u16,
    /// Whether capacity restrictions should be ignored when the server supports it.
    pub ignore_capacity: bool,
    /// Whether bought items should be delivered inside backpacks when available.
    pub in_backpacks: bool,
}

impl Decodable for PurchaseNpcShop {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            item_id: bytes.get_u16()?,
            count: bytes.get_u8()?,
            amount: bytes.get_u16()?,
            ignore_capacity: bytes.get_bool()?,
            in_backpacks: bytes.get_bool()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_purchase_npc_shop() {
        let mut payload: &[u8] = &[0x34, 0x12, 7, 0x02, 0x00, 1, 0];
        let packet = PurchaseNpcShop::decode(PacketKind::PurchaseNpcShop, &mut payload).unwrap();
        assert!(packet.ignore_capacity);
        assert!(!packet.in_backpacks);
    }
}
