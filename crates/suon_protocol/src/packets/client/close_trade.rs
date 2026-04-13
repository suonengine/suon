//! Client close-trade packet.

use super::prelude::*;

/// Packet sent by the client to close the current trade without payload data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseTrade;

impl Decodable for CloseTrade {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(CloseTrade)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_close_trade_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = CloseTrade::decode(PacketKind::CloseTrade, &mut payload)
            .expect("CloseTrade packets should decode without payload bytes");

        assert!(matches!(packet, CloseTrade));
        assert!(
            payload.is_empty(),
            "CloseTrade decoding should not consume any payload bytes"
        );
    }
}
