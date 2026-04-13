//! Client close-imbuing-window packet.

use super::prelude::*;

/// Sent by the client to close the imbuing window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseImbuingWindow;

impl Decodable for CloseImbuingWindow {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_close_imbuing_window() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            CloseImbuingWindow::decode(PacketKind::CloseImbuingWindow, &mut payload).unwrap(),
            CloseImbuingWindow
        ));
    }
}
