//! Client open-reward-wall packet.

use super::prelude::*;

/// Packet sent by the client to request the current daily reward wall state.
///
/// On the wire this is only an opcode with no trailing payload. It is used to
/// trigger the reward-wall response sequence from the server.
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
