//! Client submit-text-window packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitTextWindowPacket {
    pub window_text_id: u32,
    pub text: String,
}

impl Decodable for SubmitTextWindowPacket {
    const KIND: PacketKind = PacketKind::SubmitTextWindow;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            window_text_id: bytes.get_u32()?,
            text: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_submit_text_window() {
        let mut payload: &[u8] = &[1, 0, 0, 0, 4, 0, b't', b'e', b'x', b't'];
        let packet = SubmitTextWindowPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.text, "text");
    }
}
