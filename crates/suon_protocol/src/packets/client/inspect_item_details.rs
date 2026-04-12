//! Client inspect-item-details packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to inspect additional item details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectItemDetailsPacket {
    /// Item id being inspected.
    pub item_id: u16,

    /// Optional item tier, present for upgraded items.
    pub tier: Option<u8>,
}

impl Decodable for InspectItemDetailsPacket {
    const KIND: PacketKind = PacketKind::InspectItemDetails;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let item_id = bytes.get_u16()?;
        let tier = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u8()?)
        };

        Ok(Self { item_id, tier })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_inspect_item_details_without_tier() {
        let mut payload: &[u8] = &[0x34, 0x12];

        let packet = InspectItemDetailsPacket::decode(&mut payload)
            .expect("InspectItemDetails packets should decode base item ids");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.tier, None);
    }

    #[test]
    fn should_decode_inspect_item_details_with_tier() {
        let mut payload: &[u8] = &[0x34, 0x12, 5];

        let packet = InspectItemDetailsPacket::decode(&mut payload)
            .expect("InspectItemDetails packets should decode optional tier values");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.tier, Some(5));
        assert!(payload.is_empty());
    }
}
