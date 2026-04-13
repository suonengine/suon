//! Client set-typing-state packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to update its typing-indicator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTypingState {
    /// Raw typing-state value sent by the client.
    pub state: u8,
}

impl Decodable for SetTypingState {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            state: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_typing_state() {
        let mut payload: &[u8] = &[1];
        let packet = SetTypingState::decode(PacketKind::SetTypingState, &mut payload)
            .expect("SetTypingState packets should decode the raw state byte");
        assert_eq!(packet.state, 1);
        assert!(payload.is_empty());
    }
}
