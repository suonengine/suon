//! Client enter-game packet.

use super::prelude::*;

/// Sent by the client after a successful login to enter the game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnterGamePacket;

impl Decodable for EnterGamePacket {
    const KIND: PacketKind = PacketKind::EnterGame;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            EnterGamePacket::decode(&mut payload).unwrap(),
            EnterGamePacket
        ));
    }
}
