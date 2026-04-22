//! Client market-browse packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to browse market data.
///
/// The `request_kind` identifies the kind of market request. Some request kinds
/// are standalone, while others are followed by an `item_id`.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[3, 0x2A, 0x00];
/// let packet = BrowseMarketPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.request_kind, MarketBrowseKind::Item { item_id: 42 });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketBrowseKind {
    /// Requests the caller's own active offers.
    OwnOffers,

    /// Requests the caller's own market history.
    OwnHistory,

    /// Requests offers for a specific item.
    Item { item_id: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseMarketPacket {
    /// Identifies the requested market browse action.
    pub request_kind: MarketBrowseKind,
}

impl Decodable for BrowseMarketPacket {
    const KIND: PacketKind = PacketKind::BrowseMarket;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let request_kind = match bytes.get_u8()? {
            0 => MarketBrowseKind::OwnOffers,
            1 => MarketBrowseKind::OwnHistory,
            _ => MarketBrowseKind::Item {
                item_id: bytes.get_u16()?,
            },
        };

        Ok(Self { request_kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_browse_market_without_item_id() {
        let mut payload: &[u8] = &[0x00];

        let packet = BrowseMarketPacket::decode(&mut payload)
            .expect("BrowseMarket packets should decode browse-only payloads");

        assert_eq!(packet.request_kind, MarketBrowseKind::OwnOffers);
        assert!(
            payload.is_empty(),
            "Browse-only market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_browse_market_with_item_id() {
        let mut payload: &[u8] = &[0x03, 0x2A, 0x00];

        let packet = BrowseMarketPacket::decode(&mut payload)
            .expect("BrowseMarket packets should decode item-browse payloads");

        assert_eq!(packet.request_kind, MarketBrowseKind::Item { item_id: 42 });
        assert!(
            payload.is_empty(),
            "Item-browse market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_market_browse_kind_from_wire_value() {
        let mut own_offers: &[u8] = &[0];
        let mut own_history: &[u8] = &[1];
        let mut item: &[u8] = &[9, 0x2A, 0x00];

        assert!(matches!(
            BrowseMarketPacket::decode(&mut own_offers),
            Ok(BrowseMarketPacket {
                request_kind: MarketBrowseKind::OwnOffers
            })
        ));
        assert!(matches!(
            BrowseMarketPacket::decode(&mut own_history),
            Ok(BrowseMarketPacket {
                request_kind: MarketBrowseKind::OwnHistory
            })
        ));
        assert!(matches!(
            BrowseMarketPacket::decode(&mut item),
            Ok(BrowseMarketPacket {
                request_kind: MarketBrowseKind::Item { item_id: 42 }
            })
        ));
    }

    #[test]
    fn should_expose_browse_market_kind_constant() {
        assert_eq!(
            BrowseMarketPacket::KIND,
            PacketKind::BrowseMarket,
            "BrowseMarket packets should advertise the correct packet kind"
        );
    }
}
