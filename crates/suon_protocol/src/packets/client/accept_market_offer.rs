//! Client market-accept-offer packet.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to accept an existing market offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptMarketOffer {
    /// Timestamp that identifies the offer.
    pub timestamp: SystemTime,

    /// Counter paired with the timestamp to identify the offer.
    pub offer_counter: u16,

    /// Amount accepted from the market offer.
    pub amount: u16,
}

impl Decodable for AcceptMarketOffer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            timestamp: UNIX_EPOCH + Duration::from_secs(u64::from(bytes.get_u32()?)),
            offer_counter: bytes.get_u16()?,
            amount: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_accept_market_offer() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0x34, 0x12, 0x05, 0x00];

        let packet = AcceptMarketOffer::decode(PacketKind::AcceptMarketOffer, &mut payload)
            .expect("AcceptMarketOffer packets should decode timestamp, counter, and amount");

        assert_eq!(
            packet
                .timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            0x12345678
        );
        assert_eq!(packet.offer_counter, 0x1234);
        assert_eq!(packet.amount, 5);
        assert!(
            payload.is_empty(),
            "AcceptMarketOffer decoding should consume the whole payload"
        );
    }
}
