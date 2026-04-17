//! Client browse-market packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to browse market data.
///
/// The leading `browse_id` selects the market browse action. Own-offers and
/// own-history requests stop there, while item-browse requests append an
/// `item_id` and, for upgraded items, an `item_tier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseMarket {
    /// Selector byte that identifies which market browse flow is being used.
    pub id: u8,

    /// Item id carried by item-browse requests.
    pub item_id: Option<u16>,

    /// Optional item tier appended after `item_id` for upgraded items.
    pub item_tier: Option<u8>,
}

impl Decodable for BrowseMarket {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let browse_id = bytes.get_u8()?;
        let item_id = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u16()?)
        };

        let item_tier = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u8()?)
        };

        Ok(Self {
            id: browse_id,
            item_id,
            item_tier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_browse_market_without_item_data() {
        let mut payload: &[u8] = &[0x00];

        let packet = BrowseMarket::decode(PacketKind::BrowseMarket, &mut payload)
            .expect("BrowseMarket packets should decode browse-only payloads");

        assert_eq!(packet.id, 0x00);
        assert_eq!(packet.item_id, None);
        assert_eq!(packet.item_tier, None);
        assert!(
            payload.is_empty(),
            "Browse-only market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_browse_market_with_item_id() {
        let mut payload: &[u8] = &[0x03, 0x2A, 0x00];

        let packet = BrowseMarket::decode(PacketKind::BrowseMarket, &mut payload)
            .expect("BrowseMarket packets should decode item-browse payloads");

        assert_eq!(packet.id, 0x03);
        assert_eq!(packet.item_id, Some(42));
        assert_eq!(packet.item_tier, None);
        assert!(
            payload.is_empty(),
            "Item-browse market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_browse_market_with_item_tier() {
        let mut payload: &[u8] = &[0x03, 0x2A, 0x00, 7];

        let packet = BrowseMarket::decode(PacketKind::BrowseMarket, &mut payload)
            .expect("BrowseMarket packets should decode item tiers when present");

        assert_eq!(packet.id, 0x03);
        assert_eq!(packet.item_id, Some(42));
        assert_eq!(packet.item_tier, Some(7));
        assert!(
            payload.is_empty(),
            "Tiered item-browse market requests should consume the whole payload"
        );
    }
}
