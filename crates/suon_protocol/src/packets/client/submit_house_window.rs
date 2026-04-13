//! Client submit-house-window packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitHouseWindow {
    pub door_id: u8,
    pub house_id: u32,
    pub text: String,
}

impl Decodable for SubmitHouseWindow {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            door_id: bytes.get_u8()?,
            house_id: bytes.get_u32()?,
            text: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_submit_house_window() {
        let mut payload: &[u8] = &[7, 1, 0, 0, 0, 4, 0, b't', b'e', b'x', b't'];
        let packet =
            SubmitHouseWindow::decode(PacketKind::SubmitHouseWindow, &mut payload).unwrap();
        assert_eq!(packet.door_id, 7);
    }
}
