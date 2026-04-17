//! Client remove-imbuement packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to remove an imbuement slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoveImbuement {
    /// Equipment slot whose imbuement should be removed.
    pub slot: u8,
}

impl Decodable for RemoveImbuement {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            slot: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_clear_imbuement() {
        let mut payload: &[u8] = &[4];

        let packet = RemoveImbuement::decode(PacketKind::RemoveImbuement, &mut payload)
            .expect("RemoveImbuement packets should decode the equipment slot");

        assert_eq!(packet.slot, 4);
        assert!(payload.is_empty());
    }
}
