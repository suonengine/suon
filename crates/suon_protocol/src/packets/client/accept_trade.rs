//! Client accept-trade packet.

use super::prelude::*;

/// Packet sent by the client to accept the current trade without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::AcceptTradePacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = AcceptTradePacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, AcceptTradePacket));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptTradePacket;

impl Decodable for AcceptTradePacket {
    const KIND: PacketKind = PacketKind::AcceptTrade;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(AcceptTradePacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_accept_trade_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = AcceptTradePacket::decode(&mut payload)
            .expect("AcceptTrade packets should decode without payload bytes");

        assert!(matches!(packet, AcceptTradePacket));
        assert!(
            payload.is_empty(),
            "AcceptTrade decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_accept_trade_kind_constant() {
        assert_eq!(
            AcceptTradePacket::KIND,
            PacketKind::AcceptTrade,
            "AcceptTrade packets should advertise the correct packet kind"
        );
    }
}
