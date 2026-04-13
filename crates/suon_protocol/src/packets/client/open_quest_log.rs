//! Client open-quest-log packet.

use super::prelude::*;

/// Packet sent by the client to request the quest log listing.
///
/// This opcode is bodyless on the wire and acts as a command for the server to
/// respond with the current quest summary for the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenQuestLog;

impl Decodable for OpenQuestLog {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_quest_log() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenQuestLog::decode(PacketKind::OpenQuestLog, &mut payload).unwrap(),
            OpenQuestLog
        ));
    }
}
