//! Client set-fight-modes packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FightMode {
    Offensive = 1,
    Balanced = 2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaseMode {
    Stand = 0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecureMode {
    Safe = 0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateFightModesPacket {
    pub fight_mode: FightMode,
    pub chase_mode: ChaseMode,
    pub secure_mode: SecureMode,
}

impl Decodable for UpdateFightModesPacket {
    const KIND: PacketKind = PacketKind::UpdateFightModes;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
        let packet = UpdateFightModesPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.fight_mode, FightMode::Balanced);
        assert_eq!(packet.chase_mode, ChaseMode::Chase);
        assert_eq!(packet.secure_mode, SecureMode::Safe);
    }

    #[test]
    fn should_reject_unknown_fight_mode() {
        let mut payload: &[u8] = &[4, 1, 0];

        assert!(matches!(
            UpdateFightModesPacket::decode(&mut payload),
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
            UpdateFightModesPacket::decode(&mut payload),
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
            UpdateFightModesPacket::decode(&mut payload),
            Err(DecodableError::InvalidFieldValue {
                field: "secure_mode",
                value: 2,
            })
        ));
    }
}
