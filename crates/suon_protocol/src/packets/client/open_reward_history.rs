//! Client open-reward-history packet.

use super::prelude::*;

/// Packet sent by the client to request the daily reward history data.
///
/// The packet has no embedded fields. The server interprets the opcode as a
/// request to send back the reward history content for the account or
/// character, depending on protocol state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenRewardHistory;

impl Decodable for OpenRewardHistory {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_reward_history() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenRewardHistory::decode(PacketKind::OpenRewardHistory, &mut payload).unwrap(),
            OpenRewardHistory
        ));
    }
}
