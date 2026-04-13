//! Client accept-trade packet.

use super::prelude::*;

/// Packet sent by the client to accept the current trade without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::AcceptTrade};
///
/// let mut payload: &[u8] = &[];
/// let packet = AcceptTrade::decode(PacketKind::AcceptTrade, &mut payload).unwrap();
///
/// assert!(matches!(packet, AcceptTrade));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcceptTrade;

impl Decodable for AcceptTrade {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(AcceptTrade)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_accept_trade_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = AcceptTrade::decode(PacketKind::AcceptTrade, &mut payload)
            .expect("AcceptTrade packets should decode without payload bytes");

        assert!(matches!(packet, AcceptTrade));
        assert!(
            payload.is_empty(),
            "AcceptTrade decoding should not consume any payload bytes"
        );
    }
}
