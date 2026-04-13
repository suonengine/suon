//! Client configure-boss-slot packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to assign a boss race to a bosstiary slot.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::ConfigureBossSlot};
///
/// let mut payload: &[u8] = &[2, 0x78, 0x56, 0x34, 0x12];
/// let packet = ConfigureBossSlot::decode(PacketKind::ConfigureBossSlot, &mut payload).unwrap();
///
/// assert_eq!(packet.slot, 2);
/// assert_eq!(packet.boss_race_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigureBossSlot {
    /// Bosstiary slot being configured.
    pub slot: u8,

    /// Boss race id assigned to the slot, or `0` to clear it.
    pub boss_race_id: u32,
}

impl Decodable for ConfigureBossSlot {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            slot: bytes.get_u8()?,
            boss_race_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_configure_boss_slot() {
        let mut payload: &[u8] = &[2, 0x78, 0x56, 0x34, 0x12];

        let packet = ConfigureBossSlot::decode(PacketKind::ConfigureBossSlot, &mut payload)
            .expect("ConfigureBossSlot packets should decode slot and boss race id");

        assert_eq!(packet.slot, 2);
        assert_eq!(packet.boss_race_id, 0x12345678);
        assert!(payload.is_empty());
    }
}
