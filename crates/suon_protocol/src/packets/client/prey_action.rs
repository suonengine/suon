//! Client prey-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Prey action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreyActionKind {
    /// Rerolls the prey list for a slot.
    ListReroll,
    /// Rerolls the prey bonus for a slot.
    BonusReroll,
    /// Selects a monster by list index.
    MonsterSelection {
        /// Index chosen inside the current monster list.
        index: u8,
    },
    /// Opens the full list using prey cards.
    ListAllCards,
    /// Selects a race from the full list.
    ListAllSelection {
        /// Race id selected from the full list.
        race_id: u16,
    },
    /// Updates a prey option toggle.
    Option {
        /// Option identifier toggled by the client.
        option: u8,
    },
}

/// Packet sent by the client to perform a prey action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreyAction {
    /// Prey slot being updated.
    pub slot: u8,

    /// Action requested for the prey slot.
    pub action: PreyActionKind,
}

impl Decodable for PreyAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let slot = bytes.get_u8()?;
        let action = match bytes.get_u8()? {
            0 => PreyActionKind::ListReroll,
            1 => PreyActionKind::BonusReroll,
            2 => PreyActionKind::MonsterSelection {
                index: bytes.get_u8()?,
            },
            3 => PreyActionKind::ListAllCards,
            4 => PreyActionKind::ListAllSelection {
                race_id: bytes.get_u16()?,
            },
            5 => PreyActionKind::Option {
                option: bytes.get_u8()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
                    value,
                });
            }
        };

        Ok(Self { slot, action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_prey_monster_selection() {
        let mut payload: &[u8] = &[1, 2, 9];

        let packet = PreyAction::decode(PacketKind::PreyAction, &mut payload)
            .expect("PreyAction packets should decode monster-selection actions");

        assert_eq!(packet.slot, 1);
        assert_eq!(packet.action, PreyActionKind::MonsterSelection { index: 9 });
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_prey_option_change() {
        let mut payload: &[u8] = &[2, 5, 1];

        let packet = PreyAction::decode(PacketKind::PreyAction, &mut payload)
            .expect("PreyAction packets should decode option changes");

        assert_eq!(packet.action, PreyActionKind::Option { option: 1 });
    }
}
