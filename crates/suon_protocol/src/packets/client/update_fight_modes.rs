//! Client set-fight-modes packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Combat-stance byte used by [`UpdateFightModes`] to bias attack and defense.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FightMode {
    /// Prioritizes offensive pressure (wire value `1`).
    Offensive = 1,
    /// Keeps the neutral default stance (wire value `2`).
    Balanced = 2,
    /// Prioritizes defensive resilience (wire value `3`).
    Defensive = 3,
}

impl TryFrom<u8> for FightMode {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Offensive),
            2 => Ok(Self::Balanced),
            3 => Ok(Self::Defensive),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "fight_mode",
                value,
            }),
        }
    }
}

/// Follow behavior byte used by [`UpdateFightModes`] while attacking a creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaseMode {
    /// Keeps the character in place instead of following the target (wire value `0`).
    Stand = 0,
    /// Allows the character to chase the current target (wire value `1`).
    Chase = 1,
}

impl TryFrom<u8> for ChaseMode {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Stand),
            1 => Ok(Self::Chase),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "chase_mode",
                value,
            }),
        }
    }
}

/// Safety flag byte carried by [`UpdateFightModes`] for protected combat actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureMode {
    /// Keeps secure mode enabled (wire value `0`).
    Safe = 0,
    /// Disables secure restrictions for subsequent attacks (wire value `1`).
    Unrestricted = 1,
}

impl TryFrom<u8> for SecureMode {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Safe),
            1 => Ok(Self::Unrestricted),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "secure_mode",
                value,
            }),
        }
    }
}

/// Packet sent by the client to update its combat stance flags.
///
/// The three payload bytes encode the selected fight, chase, and secure modes
/// that should become active for subsequent combat interactions.
///
/// # Examples
///
/// ```rust
/// use suon_protocol::packets::client::prelude::{
///     ChaseMode, Decodable, FightMode, PacketKind, SecureMode, UpdateFightModes,
/// };
///
/// let mut payload: &[u8] = &[2, 1, 0];
/// let packet = UpdateFightModes::decode(PacketKind::UpdateFightModes, &mut payload).unwrap();
///
/// assert_eq!(packet.fight_mode, FightMode::Balanced);
/// assert_eq!(packet.chase_mode, ChaseMode::Chase);
/// assert_eq!(packet.secure_mode, SecureMode::Safe);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateFightModes {
    /// Selected combat stance byte for offensive versus defensive bias.
    pub fight_mode: FightMode,
    /// Selected follow behavior while attacking a target.
    pub chase_mode: ChaseMode,
    /// Selected safety restriction mode for subsequent attacks.
    pub secure_mode: SecureMode,
}

impl Decodable for UpdateFightModes {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            fight_mode: bytes.get_u8()?.try_into()?,
            chase_mode: bytes.get_u8()?.try_into()?,
            secure_mode: bytes.get_u8()?.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_update_fight_modes() {
        let mut payload: &[u8] = &[2, 1, 0];
        let packet = UpdateFightModes::decode(PacketKind::UpdateFightModes, &mut payload).unwrap();
        assert_eq!(packet.fight_mode, FightMode::Balanced);
        assert_eq!(packet.chase_mode, ChaseMode::Chase);
        assert_eq!(packet.secure_mode, SecureMode::Safe);
    }

    #[test]
    fn should_reject_unknown_fight_mode() {
        let mut payload: &[u8] = &[4, 1, 0];

        assert!(matches!(
            UpdateFightModes::decode(PacketKind::UpdateFightModes, &mut payload),
            Err(DecodableError::InvalidFieldValue {
                field: "fight_mode",
                value: 4,
            })
        ));
    }

    #[test]
    fn should_reject_unknown_chase_mode() {
        let mut payload: &[u8] = &[2, 2, 0];

        assert!(matches!(
            UpdateFightModes::decode(PacketKind::UpdateFightModes, &mut payload),
            Err(DecodableError::InvalidFieldValue {
                field: "chase_mode",
                value: 2,
            })
        ));
    }

    #[test]
    fn should_reject_unknown_secure_mode() {
        let mut payload: &[u8] = &[2, 1, 2];

        assert!(matches!(
            UpdateFightModes::decode(PacketKind::UpdateFightModes, &mut payload),
            Err(DecodableError::InvalidFieldValue {
                field: "secure_mode",
                value: 2,
            })
        ));
    }
}
