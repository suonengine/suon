//! Client open-reward-history packet.

use super::prelude::*;

/// Sent by the client to open the daily reward history window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenRewardHistoryPacket;

impl Decodable for OpenRewardHistoryPacket {
    const KIND: PacketKind = PacketKind::OpenRewardHistory;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            OpenRewardHistoryPacket::decode(&mut payload).unwrap(),
            OpenRewardHistoryPacket
        ));
    }
}
