//! Client enter-game packet.

use super::prelude::*;

/// Sent by the client after a successful login to enter the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnterGame;

impl Decodable for EnterGame {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_enter_game() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            EnterGame::decode(PacketKind::EnterGame, &mut payload).unwrap(),
            EnterGame
        ));
    }
}
