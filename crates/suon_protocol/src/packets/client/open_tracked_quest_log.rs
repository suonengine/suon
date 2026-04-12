//! Client open-tracked-quest-log packet.

use super::prelude::*;

/// Packet sent by the client to open the tracked quest log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTrackedQuestLogPacket;

impl Decodable for OpenTrackedQuestLogPacket {
    const KIND: PacketKind = PacketKind::OpenTrackedQuestLog;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_tracked_quest_log() {
        let mut payload: &[u8] = &[];

        let packet = OpenTrackedQuestLogPacket::decode(&mut payload)
            .expect("OpenTrackedQuestLog packets should decode empty payloads");

        assert!(matches!(packet, OpenTrackedQuestLogPacket));
        assert!(payload.is_empty());
    }
}
