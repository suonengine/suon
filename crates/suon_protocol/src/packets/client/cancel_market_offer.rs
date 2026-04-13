//! Client market-cancel-offer packet.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to cancel an existing market offer.
///
/// # Examples
/// ```
/// use std::time::UNIX_EPOCH;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::CancelMarketOffer};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0x34, 0x12];
/// let packet = CancelMarketOffer::decode(PacketKind::CancelMarketOffer, &mut payload).unwrap();
///
/// assert_eq!(
///     packet.timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs(),
///     0x12345678
/// );
/// assert_eq!(packet.offer_counter, 0x1234);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelMarketOffer {
    /// Timestamp that identifies the offer.
    pub timestamp: SystemTime,

    /// Counter paired with the timestamp to identify the offer.
    pub offer_counter: u16,
}

impl Decodable for CancelMarketOffer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            timestamp: UNIX_EPOCH + Duration::from_secs(u64::from(bytes.get_u32()?)),
            offer_counter: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_cancel_market_offer() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0x34, 0x12];

        let packet = CancelMarketOffer::decode(PacketKind::CancelMarketOffer, &mut payload)
            .expect("CancelMarketOffer packets should decode timestamp and counter");

        assert_eq!(
            packet
                .timestamp
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            0x12345678
        );
        assert_eq!(packet.offer_counter, 0x1234);
        assert!(
            payload.is_empty(),
            "CancelMarketOffer decoding should consume the whole payload"
        );
    }
}
