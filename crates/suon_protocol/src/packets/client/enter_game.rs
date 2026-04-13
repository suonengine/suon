//! Client enter-game packet.

use super::prelude::*;

/// Packet sent by the client to confirm character entry after login.
///
/// The opcode has no body. Once received, the server may transition the
/// connection from the login flow into the in-game protocol state.
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
        let packet = EnterGame::decode(PacketKind::EnterGame, &mut payload)
            .expect("EnterGame packets should decode empty payloads");

        assert!(matches!(packet, EnterGame));
        assert!(payload.is_empty());
    }
}
