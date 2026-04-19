//! Client market-create-offer packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// The kind of market offer being created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketOfferKind {
    /// A buy offer.
    Buy = 0,
    /// A sell offer.
    Sell = 1,
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

/// Packet sent by the client to create a market offer.
///
/// The protocol may include an optional `item_tier` byte between `item_id` and
/// `amount`. This decoder infers its presence from the remaining payload size:
/// 12 bytes means the tier byte is present, 11 bytes means it is absent.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[
///     1, 0x2A, 0x00, 10, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 1,
/// ];
/// let packet = CreateMarketOfferPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.offer_kind, MarketOfferKind::Sell);
/// assert_eq!(packet.item_id, 42);
/// assert_eq!(packet.item_tier, Some(10));
/// assert_eq!(packet.amount, 5);
/// assert_eq!(packet.price, 123_456_789);
/// assert!(packet.is_anonymous);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateMarketOfferPacket {
    /// Market offer type sent by the client.
    pub offer_kind: MarketOfferKind,

    /// Item id of the traded item as sent by the client payload.
    pub item_id: u16,

    /// Optional item tier byte for classified items.
    pub item_tier: Option<u8>,

    /// Number of items in the offer.
    pub amount: u16,

    /// Price associated with the offer.
    pub price: u64,

    /// Whether the offer should be anonymous.
    pub is_anonymous: bool,
}

impl Decodable for CreateMarketOfferPacket {
    const KIND: PacketKind = PacketKind::CreateMarketOffer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let offer_kind = MarketOfferKind::try_from(bytes.get_u8()?)?;
        let item_id = bytes.get_u16()?;

        let item_tier = match bytes.len() {
            12 => Some(bytes.get_u8()?),
            _ => None,
        };

        let amount = bytes.get_u16()?;
        let price = bytes.get_u64()?;
        let is_anonymous = bytes.get_bool()?;

        Ok(Self {
            offer_kind,
            item_id,
            item_tier,
            amount,
            price,
            is_anonymous,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_create_market_offer_without_item_tier() {
        let mut payload: &[u8] = &[1, 0x2A, 0x00, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 1];

        let packet = CreateMarketOfferPacket::decode(&mut payload)
            .expect("CreateMarketOffer packets should decode payloads without item tiers");

        assert_eq!(packet.offer_kind, MarketOfferKind::Sell);
        assert_eq!(packet.item_id, 42);
        assert_eq!(packet.item_tier, None);
        assert_eq!(packet.amount, 5);
        assert_eq!(packet.price, 123_456_789);
        assert!(packet.is_anonymous);
        assert!(
            payload.is_empty(),
            "Tierless create-market-offer packets should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_create_market_offer_with_item_tier() {
        let mut payload: &[u8] = &[
            1, 0x2A, 0x00, 10, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 0,
        ];

        let packet = CreateMarketOfferPacket::decode(&mut payload)
            .expect("CreateMarketOffer packets should decode payloads with item tiers");

        assert_eq!(packet.offer_kind, MarketOfferKind::Sell);
        assert_eq!(packet.item_id, 42);
        assert_eq!(packet.item_tier, Some(10));
        assert_eq!(packet.amount, 5);
        assert_eq!(packet.price, 123_456_789);
        assert!(!packet.is_anonymous);
        assert!(
            payload.is_empty(),
            "Tiered create-market-offer packets should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_create_market_offer_kind_constant() {
        assert_eq!(
            CreateMarketOfferPacket::KIND,
            PacketKind::CreateMarketOffer,
            "CreateMarketOffer packets should advertise the correct packet kind"
        );
    }

    #[test]
    fn should_reject_unknown_market_offer_kind() {
        let mut payload: &[u8] = &[9, 0x2A, 0x00, 5, 0, 0x15, 0xCD, 0x5B, 0x07, 0, 0, 0, 0, 1];

        let error = CreateMarketOfferPacket::decode(&mut payload)
            .expect_err("CreateMarketOffer packets should reject unknown offer kinds");

        assert!(
            matches!(
                error,
                DecodableError::InvalidFieldValue {
                    field: "offer_kind",
                    value: 9
                }
            ),
            "Unknown market-offer kinds should surface a typed decoding error"
        );
    }
}
