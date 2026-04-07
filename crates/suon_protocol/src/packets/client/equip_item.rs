//! Client equip-item packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to equip an item directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EquipItemPacket {
    /// Item id used by the client protocol.
    pub item_id: u16,
}

impl Decodable for EquipItemPacket {
    const KIND: PacketKind = PacketKind::EquipItem;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            item_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_equip_item() {
        let mut payload: &[u8] = &[0x34, 0x12];
        let packet = EquipItemPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.item_id, 0x1234);
    }
}
