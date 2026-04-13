//! Client open-bless-dialog packet.

use super::prelude::*;

/// Packet sent by the client to request available blessings for the current
/// shrine interaction.
///
/// The packet has no payload. On the wire it acts as a pure command, asking
/// the server to evaluate the player's state and return the blessing data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBlessDialog;

impl Decodable for OpenBlessDialog {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_bless_dialog() {
        let mut payload: &[u8] = &[];
        let packet = OpenBlessDialog::decode(PacketKind::OpenBlessDialog, &mut payload)
            .expect("OpenBlessDialog packets should decode empty payloads");

        assert!(matches!(packet, OpenBlessDialog));
        assert!(payload.is_empty());
    }
}
