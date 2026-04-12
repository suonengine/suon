//! Client close-imbuing-window packet.

use super::prelude::*;

/// Sent by the client to close the imbuing window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseImbuingWindowPacket;

impl Decodable for CloseImbuingWindowPacket {
    const KIND: PacketKind = PacketKind::CloseImbuingWindow;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            CloseImbuingWindowPacket::decode(&mut payload).unwrap(),
            CloseImbuingWindowPacket
        ));
    }
}
