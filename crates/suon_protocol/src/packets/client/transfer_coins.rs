//! Client transfer-coins packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to transfer transferable coins to another character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferCoinsPacket {
    /// Recipient character name.
    pub recipient: String,

    /// Coin amount requested by the client.
    pub amount: u32,
}

impl Decodable for TransferCoinsPacket {
    const KIND: PacketKind = PacketKind::TransferCoins;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            recipient: bytes.get_string()?,
            amount: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_coin_transfer() {
        let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n', 0x78, 0x56, 0x34, 0x12];

        let packet = TransferCoinsPacket::decode(&mut payload)
            .expect("TransferCoins packets should decode recipient and amount");

        assert_eq!(packet.recipient, "John");
        assert_eq!(packet.amount, 0x12345678);
        assert!(payload.is_empty());
    }
}
