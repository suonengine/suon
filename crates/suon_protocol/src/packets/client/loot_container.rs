//! Client loot-container packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Managed loot-container action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LootContainerAction {
    /// Assigns a container to a category using fallback mode.
    SetFallbackContainer {
        /// Category being configured.
        category: u8,
        /// Position of the container item.
        position: Position,
        /// Item id of the container item.
        item_id: u16,
        /// Stack position of the container item.
        stack_position: u8,
    },
    /// Clears a fallback container category.
    ClearFallbackContainer {
        /// Category being cleared.
        category: u8,
    },
    /// Opens a fallback container category.
    OpenFallbackContainer {
        /// Category being opened.
        category: u8,
    },
    /// Sets the quick-loot fallback behavior.
    SetMainFallback {
        /// Whether the main backpack should be used as fallback.
        use_main_as_fallback: bool,
    },
    /// Assigns a primary managed container.
    SetPrimaryContainer {
        /// Category being configured.
        category: u8,
        /// Position of the container item.
        position: Position,
        /// Item id of the container item.
        item_id: u16,
        /// Stack position of the container item.
        stack_position: u8,
    },
    /// Clears a primary managed container category.
    ClearPrimaryContainer {
        /// Category being cleared.
        category: u8,
    },
    /// Opens a primary managed container category.
    OpenPrimaryContainer {
        /// Category being opened.
        category: u8,
    },
}

/// Packet sent by the client to manage quick-loot containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LootContainer {
    /// Action requested by the client.
    pub action: LootContainerAction,
}

impl Decodable for LootContainer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => LootContainerAction::SetFallbackContainer {
                category: bytes.get_u8()?,
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            1 => LootContainerAction::ClearFallbackContainer {
                category: bytes.get_u8()?,
            },
            2 => LootContainerAction::OpenFallbackContainer {
                category: bytes.get_u8()?,
            },
            3 => LootContainerAction::SetMainFallback {
                use_main_as_fallback: bytes.get_bool()?,
            },
            4 => LootContainerAction::SetPrimaryContainer {
                category: bytes.get_u8()?,
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            5 => LootContainerAction::ClearPrimaryContainer {
                category: bytes.get_u8()?,
            },
            6 => LootContainerAction::OpenPrimaryContainer {
                category: bytes.get_u8()?,
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
    fn should_decode_set_primary_loot_container() {
        let mut payload: &[u8] = &[4, 2, 0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A, 7];

        let packet = LootContainer::decode(PacketKind::LootContainer, &mut payload)
            .expect("LootContainer packets should decode set-container actions");

        assert_eq!(
            packet.action,
            LootContainerAction::SetPrimaryContainer {
                category: 2,
                position: Position {
                    x: 0x1234,
                    y: 0x5678,
                },
                item_id: 0x9ABC,
                stack_position: 7,
            }
        );
    }
}
