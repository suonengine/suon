//! Client open-reward-history packet.

use super::prelude::*;

/// Sent by the client to open the daily reward history window.
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
