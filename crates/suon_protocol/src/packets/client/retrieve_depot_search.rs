//! Client retrieve-depot-search packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to retrieve an item from depot search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetrieveDepotSearchPacket {
    /// Item id requested by the client.
    pub item_id: u16,

    /// Optional item tier for upgraded items.
    pub item_tier: Option<u8>,

    /// Retrieval mode requested by the client.
    pub retrieval_type: u8,
}

impl Decodable for RetrieveDepotSearchPacket {
    const KIND: PacketKind = PacketKind::RetrieveDepotSearch;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let item_id = bytes.get_u16()?;
        let remaining = bytes.len();
        let (item_tier, retrieval_type) = if remaining >= 2 {
            (Some(bytes.get_u8()?), bytes.get_u8()?)
        } else {
            (None, bytes.get_u8()?)
        };

        Ok(Self {
            item_id,
            item_tier,
            retrieval_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_retrieve_depot_search_without_tier() {
        let mut payload: &[u8] = &[0x34, 0x12, 2];

        let packet = RetrieveDepotSearchPacket::decode(&mut payload)
            .expect("RetrieveDepotSearch packets should decode item id and retrieval type");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.item_tier, None);
        assert_eq!(packet.retrieval_type, 2);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_retrieve_depot_search_with_tier() {
        let mut payload: &[u8] = &[0x34, 0x12, 5, 1];

        let packet = RetrieveDepotSearchPacket::decode(&mut payload)
            .expect("RetrieveDepotSearch packets should decode optional item tier");

        assert_eq!(packet.item_tier, Some(5));
        assert_eq!(packet.retrieval_type, 1);
        assert!(payload.is_empty());
    }
}
