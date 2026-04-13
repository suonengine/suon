//! Client submit-text-window packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to submit text back to a generic text window.
///
/// The payload carries the server-provided text window id together with the new
/// string content entered or confirmed by the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitTextWindow {
    /// Server-provided text-window id being submitted back to the server.
    pub window_text_id: u32,

    /// Final text content entered or confirmed in the window.
    pub text: String,
}

impl Decodable for SubmitTextWindow {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
        let packet = SubmitTextWindow::decode(PacketKind::SubmitTextWindow, &mut payload).unwrap();
        assert_eq!(packet.text, "text");
    }
}
