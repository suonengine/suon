//! Client quick-loot packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Quick-loot action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickLootAction {
    /// Loots a single corpse at a given position.
    LootSingle {
        /// Position of the corpse being looted.
        position: Position,
        /// Item id of the corpse.
        item_id: u16,
        /// Stack position of the corpse.
        stack_position: u8,
    },
    /// Loots all corpses around the selected position.
    LootAll {
        /// Position of the reference corpse.
        position: Position,
        /// Item id of the corpse.
        item_id: u16,
        /// Stack position of the corpse.
        stack_position: u8,
    },
    /// Loots nearby corpses without a position payload.
    LootNearby,
}

/// Packet sent by the client to perform a quick-loot action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickLoot {
    /// Quick-loot action requested by the client.
    pub action: QuickLootAction,
}

impl Decodable for QuickLoot {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => QuickLootAction::LootSingle {
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            1 => QuickLootAction::LootAll {
                position: bytes.get_position()?,
                item_id: bytes.get_u16()?,
                stack_position: bytes.get_u8()?,
            },
            2 => QuickLootAction::LootNearby,
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "variant",
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
    fn should_decode_quick_loot_nearby() {
        let mut payload: &[u8] = &[2];

        let packet = QuickLoot::decode(PacketKind::QuickLoot, &mut payload)
            .expect("QuickLoot packets should decode the nearby-loot variant");

        assert_eq!(packet.action, QuickLootAction::LootNearby);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_quick_loot_all() {
        let mut payload: &[u8] = &[1, 0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A, 6];

        let packet = QuickLoot::decode(PacketKind::QuickLoot, &mut payload)
            .expect("QuickLoot packets should decode the loot-all variant");

        assert_eq!(
            packet.action,
            QuickLootAction::LootAll {
                position: Position {
                    x: 0x1234,
                    y: 0x5678,
                },
                item_id: 0x9ABC,
                stack_position: 6,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_quick_loot_single() {
        let mut payload: &[u8] = &[0, 0x34, 0x12, 0x78, 0x56, 0xBC, 0x9A, 6];

        let packet = QuickLoot::decode(PacketKind::QuickLoot, &mut payload)
            .expect("QuickLoot packets should decode the single-loot variant");

        assert_eq!(
            packet.action,
            QuickLootAction::LootSingle {
                position: Position {
                    x: 0x1234,
                    y: 0x5678,
                },
                item_id: 0x9ABC,
                stack_position: 6,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_reject_unknown_quick_loot_variant() {
        let mut payload: &[u8] = &[9];

        let error = QuickLoot::decode(PacketKind::QuickLoot, &mut payload)
            .expect_err("QuickLoot packets should reject unsupported variants");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "variant",
                value: 9,
            }
        ));
    }
}
