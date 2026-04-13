//! Client open-prey-dialog packet.

use super::prelude::*;

/// Sent by the client to open the prey window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenPreyDialog;

impl Decodable for OpenPreyDialog {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_prey_dialog() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenPreyDialog::decode(PacketKind::OpenPreyDialog, &mut payload).unwrap(),
            OpenPreyDialog
        ));
    }
}
