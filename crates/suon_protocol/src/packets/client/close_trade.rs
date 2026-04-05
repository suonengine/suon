//! Client close-trade packet.

use super::prelude::*;

/// Packet sent by the client to close the current trade without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::CloseTradePacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = CloseTradePacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, CloseTradePacket));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseTradePacket;

impl Decodable for CloseTradePacket {
    const KIND: PacketKind = PacketKind::CloseTrade;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(CloseTradePacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_close_trade_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = CloseTradePacket::decode(&mut payload)
            .expect("CloseTrade packets should decode without payload bytes");

        assert!(matches!(packet, CloseTradePacket));
        assert!(
            payload.is_empty(),
            "CloseTrade decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_close_trade_kind_constant() {
        assert_eq!(
            CloseTradePacket::KIND,
            PacketKind::CloseTrade,
            "CloseTrade packets should advertise the correct packet kind"
        );
    }
}
