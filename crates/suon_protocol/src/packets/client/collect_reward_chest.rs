//! Client collect-reward-chest packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Packet sent by the client to collect rewards from a reward chest slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectRewardChest {
    /// Position of the reward chest.
    pub position: Position,

    /// Item id of the chest object.
    pub item_id: u16,

    /// Stack position of the chest object.
    pub stack_position: u8,
}

impl Decodable for CollectRewardChest {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_collect_reward_chest() {
        let mut payload: &[u8] = &[0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A, 4];

        let packet = CollectRewardChest::decode(PacketKind::CollectRewardChest, &mut payload)
            .expect("CollectRewardChest packets should decode chest position and stack info");

        assert_eq!(
            packet.position,
            Position {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert_eq!(packet.item_id, 0x9ABC);
        assert_eq!(packet.stack_position, 4);
        assert!(payload.is_empty());
    }
}
