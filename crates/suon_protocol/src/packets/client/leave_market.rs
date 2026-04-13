//! Client market-leave packet.

use super::prelude::*;

/// Packet sent by the client to leave the market flow without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::LeaveMarket};
///
/// let mut payload: &[u8] = &[];
/// let packet = LeaveMarket::decode(PacketKind::LeaveMarket, &mut payload).unwrap();
///
/// assert!(matches!(packet, LeaveMarket));
/// assert!(payload.is_empty());
/// ```
pub struct LeaveMarket;

impl Decodable for LeaveMarket {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(LeaveMarket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_market_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = LeaveMarket::decode(PacketKind::LeaveMarket, &mut payload)
            .expect("LeaveMarket packets should decode without payload bytes");

        assert!(matches!(packet, LeaveMarket));
        assert!(
            payload.is_empty(),
            "LeaveMarket decoding should not consume any payload bytes"
        );
    }
}
