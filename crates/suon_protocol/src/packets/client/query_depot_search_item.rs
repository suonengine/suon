//! Client query-depot-search-item packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to query one depot-search item entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryDepotSearchItem {
    /// Item id selected by the client.
    pub item_id: u16,

    /// Optional item tier for upgraded items.
    pub item_tier: Option<u8>,
}

impl Decodable for QueryDepotSearchItem {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let item_id = bytes.get_u16()?;
        let item_tier = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u8()?)
        };

        Ok(Self { item_id, item_tier })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_depot_search_item_query() {
        let mut payload: &[u8] = &[0x34, 0x12, 5];

        let packet = QueryDepotSearchItem::decode(PacketKind::QueryDepotSearchItem, &mut payload)
            .expect("QueryDepotSearchItem packets should decode optional tier");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.item_tier, Some(5));
        assert!(payload.is_empty());
    }
}
