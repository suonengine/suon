//! Client market-leave packet.

use super::prelude::*;

/// Packet sent by the client to close the market interface without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::LeaveMarketPacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = LeaveMarketPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, LeaveMarketPacket));
/// ```
pub struct LeaveMarketPacket;

impl Decodable for LeaveMarketPacket {
    const KIND: PacketKind = PacketKind::LeaveMarket;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(LeaveMarketPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_market_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = LeaveMarketPacket::decode(&mut payload)
            .expect("LeaveMarket packets should decode without payload bytes");

        assert!(matches!(packet, LeaveMarketPacket));
        assert!(
            payload.is_empty(),
            "LeaveMarket decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_leave_market_kind_constant() {
        assert_eq!(
            LeaveMarketPacket::KIND,
            PacketKind::LeaveMarket,
            "LeaveMarket packets should advertise the correct packet kind"
        );
    }
}
