//! Client aim-at-target packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Spell-state pair sent by the client for the target-aim system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AimAtTargetSpell {
    /// Spell identifier.
    pub spell_id: u16,

    /// State selector for the spell entry.
    pub state: u8,
}

/// Packet sent by the client to configure target-aim spell states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AimAtTarget {
    /// Spell entries included in the payload.
    pub spells: Vec<AimAtTargetSpell>,
}

impl Decodable for AimAtTarget {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let count = bytes.get_u8()? as usize;
        let mut spells = Vec::with_capacity(count);

        for _ in 0..count {
            spells.push(AimAtTargetSpell {
                spell_id: bytes.get_u16()?,
                state: bytes.get_u8()?,
            });
        }

        Ok(Self { spells })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_aim_at_target_packet() {
        let mut payload: &[u8] = &[2, 0x34, 0x12, 1, 0x78, 0x56, 0];

        let packet = AimAtTarget::decode(PacketKind::AimAtTarget, &mut payload)
            .expect("AimAtTarget packets should decode all spell-state entries");

        assert_eq!(
            packet.spells,
            vec![
                AimAtTargetSpell {
                    spell_id: 0x1234,
                    state: 1,
                },
                AimAtTargetSpell {
                    spell_id: 0x5678,
                    state: 0,
                },
            ]
        );
        assert!(payload.is_empty());
    }
}
