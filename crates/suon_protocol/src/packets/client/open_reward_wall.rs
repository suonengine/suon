//! Client open-reward-wall packet.

use super::prelude::*;

/// Sent by the client to open the daily reward wall.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenRewardWall;

impl Decodable for OpenRewardWall {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_reward_wall() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenRewardWall::decode(PacketKind::OpenRewardWall, &mut payload).unwrap(),
            OpenRewardWall
        ));
    }
}
