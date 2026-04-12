//! Client set-monster-podium packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Packet sent by the client to configure a monster podium entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetMonsterPodiumPacket {
    /// Monster race id to place on the podium.
    pub monster_race_id: u32,

    /// Position of the podium item.
    pub position: Position,

    /// Podium item id.
    pub item_id: u16,

    /// Stack position of the podium item.
    pub stack_position: u8,

    /// Facing direction byte for the shown monster.
    pub direction: u8,

    /// Whether the podium is visible.
    pub podium_visible: u8,

    /// Whether the monster is visible.
    pub monster_visible: u8,
}

impl Decodable for SetMonsterPodiumPacket {
    const KIND: PacketKind = PacketKind::SetMonsterPodium;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            monster_race_id: bytes.get_u32()?,
            position: bytes.get_position()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            direction: bytes.get_u8()?,
            podium_visible: bytes.get_u8()?,
            monster_visible: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_set_monster_podium() {
        let mut payload: &[u8] = &[
            0x78, 0x56, 0x34, 0x12, 0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A, 4, 2, 1, 0,
        ];

        let packet = SetMonsterPodiumPacket::decode(&mut payload)
            .expect("SetMonsterPodium packets should decode podium configuration");

        assert_eq!(packet.monster_race_id, 0x12345678);
        assert_eq!(packet.item_id, 0x9ABC);
        assert_eq!(packet.direction, 2);
        assert!(payload.is_empty());
    }
}
