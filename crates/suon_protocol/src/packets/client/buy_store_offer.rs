//! Client buy-store-offer packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to buy or redeem a store offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuyStoreOfferPacket {
    /// Store offer id being used.
    pub offer_id: u32,

    /// Action selector sent by the client.
    pub action: u8,

    /// Optional name argument used by some store actions.
    pub name: Option<String>,

    /// Optional type selector used by transfer-like store actions.
    pub transfer_type: Option<u8>,

    /// Optional destination or location string.
    pub location: Option<String>,
}

impl Decodable for BuyStoreOfferPacket {
    const KIND: PacketKind = PacketKind::BuyStoreOffer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let offer_id = bytes.get_u32()?;
        let action = bytes.get_u8()?;

        let (name, transfer_type, location) = if (1..6).contains(&action) {
            let name = Some(bytes.get_string()?);
            let transfer_type = if matches!(action, 3 | 5) {
                Some(bytes.get_u8()?)
            } else {
                None
            };
            let location = if action == 5 {
                Some(bytes.get_string()?)
            } else {
                None
            };

            (name, transfer_type, location)
        } else {
            (None, None, None)
        };

        Ok(Self {
            offer_id,
            action,
            name,
            transfer_type,
            location,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_simple_store_offer_purchase() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0];

        let packet = BuyStoreOfferPacket::decode(&mut payload)
            .expect("BuyStoreOffer packets should decode bare purchases");

        assert_eq!(packet.offer_id, 0x12345678);
        assert_eq!(packet.action, 0);
        assert_eq!(packet.name, None);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_store_offer_purchase_with_location() {
        let mut payload: &[u8] = &[
            0x78, 0x56, 0x34, 0x12, 5, 4, 0, b'J', b'o', b'h', b'n', 2, 5, 0, b'D', b'e', b'p',
            b'o', b't',
        ];

        let packet = BuyStoreOfferPacket::decode(&mut payload)
            .expect("BuyStoreOffer packets should decode optional transfer data");

        assert_eq!(packet.action, 5);
        assert_eq!(packet.name.as_deref(), Some("John"));
        assert_eq!(packet.transfer_type, Some(2));
        assert_eq!(packet.location.as_deref(), Some("Depot"));
        assert!(payload.is_empty());
    }
}
