//! Client buy-charm-rune packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to buy or assign a charm rune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuyCharmRune {
    /// Charm-rune action selected by the client.
    pub action: u8,

    /// Charm rune identifier.
    pub charm_id: u8,

    /// Race id associated with the action.
    pub race_id: u16,
}

impl Decodable for BuyCharmRune {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            action: bytes.get_u8()?,
            charm_id: bytes.get_u8()?,
            race_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_buy_charm_rune() {
        let mut payload: &[u8] = &[1, 7, 0x34, 0x12];

        let packet = BuyCharmRune::decode(PacketKind::BuyCharmRune, &mut payload)
            .expect("BuyCharmRune packets should decode action, charm id, and race id");

        assert_eq!(packet.action, 1);
        assert_eq!(packet.charm_id, 7);
        assert_eq!(packet.race_id, 0x1234);
        assert!(payload.is_empty());
    }
}
