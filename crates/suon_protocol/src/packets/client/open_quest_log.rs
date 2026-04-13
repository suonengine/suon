//! Client open-quest-log packet.

use super::prelude::*;

/// Sent by the client to open the quest log window.
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
