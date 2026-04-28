//! Client market packet family.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// The kind of market offer being created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketOfferKind {
    /// A buy offer.
    Buy,
    /// A sell offer.
    Sell,
}

impl TryFrom<u8> for MarketOfferKind {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Buy),
            1 => Ok(Self::Sell),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "offer_kind",
                value,
            }),
        }
    }
}

/// Market browse action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketBrowseKind {
    /// Requests the caller's own active offers.
    OwnOffers,

    /// Requests the caller's own market history.
    OwnHistory,

    /// Requests offers for a specific item.
    Item { item_id: u16 },
}

/// Packet sent by the client for any market action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketPacket {
    /// Closes the market interface.
    Leave,

    /// Browses market data.
    Browse { request_kind: MarketBrowseKind },

    /// Creates a market offer.
    CreateOffer {
        /// Market offer type sent by the client.
        offer_kind: MarketOfferKind,

        /// Item id of the traded item as sent by the client payload.
        item_id: u16,

        /// Optional item tier byte for classified items.
        item_tier: Option<u8>,

        /// Number of items in the offer.
        amount: u16,

        /// Price associated with the offer.
        price: u64,

        /// Whether the offer should be anonymous.
        is_anonymous: bool,
    },

    /// Cancels a market offer.
    CancelOffer {
        /// Timestamp that identifies the offer.
        timestamp: SystemTime,

        /// Counter paired with the timestamp to identify the offer.
        offer_counter: u16,
    },

    /// Accepts a market offer.
    AcceptOffer {
        /// Timestamp that identifies the offer.
        timestamp: SystemTime,

        /// Counter paired with the timestamp to identify the offer.
        offer_counter: u16,

        /// Amount accepted from the market offer.
        amount: u16,
    },
}

impl MarketPacket {
    fn decode_browse(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let request_kind = match bytes.get_u8()? {
            0 => MarketBrowseKind::OwnOffers,
            1 => MarketBrowseKind::OwnHistory,
            _ => MarketBrowseKind::Item {
                item_id: bytes.get_u16()?,
            },
        };

        Ok(Self::Browse { request_kind })
    }

    fn decode_create_offer(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let offer_kind = MarketOfferKind::try_from(bytes.get_u8()?)?;
        let item_id = bytes.get_u16()?;
        let item_tier = match bytes.len() {
            12 => Some(bytes.get_u8()?),
            _ => None,
        };
        let amount = bytes.get_u16()?;
        let price = bytes.get_u64()?;
        let is_anonymous = bytes.get_bool()?;

        Ok(Self::CreateOffer {
            offer_kind,
            item_id,
            item_tier,
            amount,
            price,
            is_anonymous,
        })
    }

    fn decode_cancel_offer(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self::CancelOffer {
            timestamp: UNIX_EPOCH + Duration::from_secs(u64::from(bytes.get_u32()?)),
            offer_counter: bytes.get_u16()?,
        })
    }

    fn decode_accept_offer(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self::AcceptOffer {
            timestamp: UNIX_EPOCH + Duration::from_secs(u64::from(bytes.get_u32()?)),
            offer_counter: bytes.get_u16()?,
            amount: bytes.get_u16()?,
        })
    }

    fn action_from_kind(
        kind: PacketKind,
    ) -> Option<fn(&mut &[u8]) -> Result<Self, DecodableError>> {
        match kind {
            PacketKind::LeaveMarket => Some(|_| Ok(Self::Leave)),
            PacketKind::BrowseMarket => Some(Self::decode_browse),
            PacketKind::CreateMarketOffer => Some(Self::decode_create_offer),
            PacketKind::CancelMarketOffer => Some(Self::decode_cancel_offer),
            PacketKind::AcceptMarketOffer => Some(Self::decode_accept_offer),
            _ => None,
        }
    }
}

impl Decodable for MarketPacket {
    const KIND: PacketKind = PacketKind::LeaveMarket;

    fn accepts_kind(kind: PacketKind) -> bool {
        Self::action_from_kind(kind).is_some()
    }

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Err(DecodableError::InvalidFieldValue {
            field: "packet_kind",
            value: Self::KIND as u8,
        })
    }

    fn decode_with_kind(kind: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let Some(decode) = Self::action_from_kind(kind) else {
            return Err(DecodableError::InvalidFieldValue {
                field: "packet_kind",
                value: kind as u8,
            });
        };

        decode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_market_browse_without_item_id() {
        let mut payload: &[u8] = &[0x00];

        let packet = MarketPacket::decode_with_kind(PacketKind::BrowseMarket, &mut payload)
            .expect("Market packets should decode browse-only payloads");

        assert_eq!(
            packet,
            MarketPacket::Browse {
                request_kind: MarketBrowseKind::OwnOffers
            }
        );
        assert!(
            payload.is_empty(),
            "Browse-only market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_market_browse_with_item_id() {
        let mut payload: &[u8] = &[0x03, 0x2A, 0x00];

        let packet = MarketPacket::decode_with_kind(PacketKind::BrowseMarket, &mut payload)
            .expect("Market packets should decode item-browse payloads");

        assert_eq!(
            packet,
            MarketPacket::Browse {
                request_kind: MarketBrowseKind::Item { item_id: 42 }
            }
        );
        assert!(
            payload.is_empty(),
            "Item-browse market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_create_market_offer_with_item_tier() {
        let mut payload: &[u8] = &[
            1, 0x2A, 0x00, 10, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 0,
        ];

        let packet = MarketPacket::decode_with_kind(PacketKind::CreateMarketOffer, &mut payload)
            .expect("Market packets should decode create-offer payloads");

        assert_eq!(
            packet,
            MarketPacket::CreateOffer {
                offer_kind: MarketOfferKind::Sell,
                item_id: 42,
                item_tier: Some(10),
                amount: 5,
                price: 123_456_789,
                is_anonymous: false,
            }
        );
        assert!(
            payload.is_empty(),
            "Create-market-offer packets should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_accept_market_offer() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0x34, 0x12, 0x05, 0x00];

        let packet = MarketPacket::decode_with_kind(PacketKind::AcceptMarketOffer, &mut payload)
            .expect("Market packets should decode accept-offer payloads");

        assert!(matches!(
            packet,
            MarketPacket::AcceptOffer {
                offer_counter: 0x1234,
                amount: 5,
                ..
            }
        ));
        assert!(
            payload.is_empty(),
            "Accept-market-offer packets should consume the whole payload"
        );
    }

    #[test]
    fn should_accept_all_market_packet_kinds() {
        assert!(MarketPacket::accepts_kind(PacketKind::LeaveMarket));
        assert!(MarketPacket::accepts_kind(PacketKind::BrowseMarket));
        assert!(MarketPacket::accepts_kind(PacketKind::CreateMarketOffer));
        assert!(MarketPacket::accepts_kind(PacketKind::CancelMarketOffer));
        assert!(MarketPacket::accepts_kind(PacketKind::AcceptMarketOffer));
        assert!(!MarketPacket::accepts_kind(PacketKind::AcceptTrade));
    }

    #[test]
    fn should_reject_unknown_market_offer_kind() {
        let mut payload: &[u8] = &[9, 0x2A, 0x00, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 1];

        let error = MarketPacket::decode_with_kind(PacketKind::CreateMarketOffer, &mut payload)
            .expect_err("Market packets should reject unknown offer kinds");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "offer_kind",
                value: 9
            }
        ));
    }
}
