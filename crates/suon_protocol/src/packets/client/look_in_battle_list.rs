//! Client look-in-battle-list packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to request the description of a creature selected from the battle list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookInBattleList {
    /// Creature identifier selected in the battle list.
    pub creature_id: u32,
}

impl Decodable for LookInBattleList {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            creature_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_look_in_battle_list() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
        assert_eq!(
            LookInBattleList::decode(PacketKind::LookInBattleList, &mut payload)
                .unwrap()
                .creature_id,
            0x12345678
        );
    }
}
