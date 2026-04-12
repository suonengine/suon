//! Client inspect-offer packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to inspect an offer description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectOfferPacket {
    /// Offer identifier requested by the client.
    pub offer_id: u32,
}

impl Decodable for InspectOfferPacket {
    const KIND: PacketKind = PacketKind::InspectOffer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            offer_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_inspect_offer() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = InspectOfferPacket::decode(&mut payload)
            .expect("InspectOffer packets should decode the offer id");

        assert_eq!(packet.offer_id, 0x12345678);
        assert!(payload.is_empty());
    }
}
