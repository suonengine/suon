//! Client stash-action packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Stash action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StashActionKind {
    /// Stows a single item amount.
    StowItem {
        /// Position of the source item.
        position: Position,
        /// Item id being stowed.
        item_id: u16,
        /// Stack position of the item.
        stack_position: u8,
        /// Count requested by the client.
        count: u8,
    },
    /// Stows all valid items from a container.
    StowContainer {
        /// Position of the source container.
        position: Position,
        /// Item id of the container.
        item_id: u16,
        /// Stack position of the container.
        stack_position: u8,
    },
    /// Stows an item stack as a whole.
    StowStack {
        /// Position of the source stack.
        position: Position,
        /// Item id being stowed.
        item_id: u16,
        /// Stack position of the item.
        stack_position: u8,
    },
    /// Withdraws an item amount from stash storage.
    Withdraw {
        /// Item id to withdraw.
        item_id: u16,
        /// Count to withdraw.
        count: u32,
        /// Destination stack position.
        stack_position: u8,
    },
}

/// Packet sent by the client to interact with the stash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StashAction {
    /// Stash action requested by the client.
    pub action: StashActionKind,
}

impl Decodable for StashAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => StashActionKind::StowItem {
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
                count: bytes.get_u8()?,
            },
            1 => StashActionKind::StowContainer {
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            2 => StashActionKind::StowStack {
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            3 => StashActionKind::Withdraw {
                item_id: bytes.get_u16()?,
                count: bytes.get_u32()?,
                stack_position: bytes.get_u8()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
                    value,
                });
            }
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_stash_withdraw() {
        let mut payload: &[u8] = &[3, 0x34, 0x12, 5, 0, 0, 0, 7];

        let packet = StashAction::decode(PacketKind::StashAction, &mut payload)
            .expect("StashAction packets should decode withdraw requests");

        assert_eq!(
            packet.action,
            StashActionKind::Withdraw {
                item_id: 0x1234,
                count: 5,
                stack_position: 7,
            }
        );
        assert!(payload.is_empty());
    }
}
