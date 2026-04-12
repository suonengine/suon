//! Client apply-imbuement packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to apply an imbuement to a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplyImbuementPacket {
    /// Equipment slot receiving the imbuement.
    pub slot: u8,

    /// Imbuement identifier selected by the client.
    pub imbuement_id: u32,

    /// Whether a protection charm should be consumed.
    pub use_protection_charm: bool,
}

impl Decodable for ApplyImbuementPacket {
    const KIND: PacketKind = PacketKind::ApplyImbuement;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            slot: bytes.get_u8()?,
            imbuement_id: bytes.get_u32()?,
            use_protection_charm: bytes.get_bool()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_apply_imbuement() {
        let mut payload: &[u8] = &[3, 0x78, 0x56, 0x34, 0x12, 1];

        let packet = ApplyImbuementPacket::decode(&mut payload)
            .expect("ApplyImbuement packets should decode slot, imbuement id, and protection flag");

        assert_eq!(packet.slot, 3);
        assert_eq!(packet.imbuement_id, 0x12345678);
        assert!(packet.use_protection_charm);
        assert!(payload.is_empty());
    }
}
