//! Client update-monster-tracker packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to enable or disable a bestiary or bosstiary tracker entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateMonsterTracker {
    /// Race id of the tracked entry.
    pub race_id: u16,

    /// Raw tracker-button state sent by the client.
    pub tracker_state: u8,
}

impl Decodable for UpdateMonsterTracker {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            race_id: bytes.get_u16()?,
            tracker_state: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_update_monster_tracker() {
        let mut payload: &[u8] = &[0x34, 0x12, 1];

        let packet = UpdateMonsterTracker::decode(PacketKind::UpdateMonsterTracker, &mut payload)
            .expect("UpdateMonsterTracker packets should decode race id and tracker state");

        assert_eq!(packet.race_id, 0x1234);
        assert_eq!(packet.tracker_state, 1);
        assert!(payload.is_empty());
    }
}
