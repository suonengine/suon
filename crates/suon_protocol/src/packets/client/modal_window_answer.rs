//! Client modal-window-answer packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client carrying the player's button and choice selection for a modal window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalWindowAnswer {
    /// Server-provided modal-window id being answered.
    pub window_id: u32,

    /// Button identifier pressed by the player.
    pub button_id: u8,

    /// Choice identifier selected by the player inside the modal window.
    pub choice_id: u8,
}

impl Decodable for ModalWindowAnswer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            window_id: bytes.get_u32()?,
            button_id: bytes.get_u8()?,
            choice_id: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_modal_window_answer() {
        let mut payload: &[u8] = &[1, 0, 0, 0, 2, 3];
        let packet = ModalWindowAnswer::decode(PacketKind::ModalWindowAnswer, &mut payload)
            .expect("ModalWindowAnswer packets should decode id, button, and choice");

        assert_eq!(packet.window_id, 1);
        assert_eq!(packet.button_id, 2);
        assert_eq!(packet.choice_id, 3);
        assert!(payload.is_empty());
    }
}
