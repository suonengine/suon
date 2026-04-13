//! Client accept-trade-offer packet.

use super::prelude::*;

/// Packet sent by the client to accept the current trade without payload data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptTradeOffer;

impl Decodable for AcceptTradeOffer {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(AcceptTradeOffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_accept_trade_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = AcceptTradeOffer::decode(PacketKind::AcceptTrade, &mut payload)
            .expect("AcceptTrade packets should decode without payload bytes");

        assert!(matches!(packet, AcceptTradeOffer));
        assert!(
            payload.is_empty(),
            "AcceptTrade decoding should not consume any payload bytes"
        );
    }
}
