//! Client modal-window-answer packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalWindowAnswer {
    pub window_id: u32,
    pub button_id: u8,
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
        let packet =
            ModalWindowAnswer::decode(PacketKind::ModalWindowAnswer, &mut payload).unwrap();
        assert_eq!(packet.choice_id, 3);
    }
}
