//! Client open-outfit-dialog packet.

use super::prelude::*;

/// Sent by the client to open the outfit selection window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenOutfitDialog;

impl Decodable for OpenOutfitDialog {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_outfit_dialog() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenOutfitDialog::decode(PacketKind::OpenOutfitDialog, &mut payload).unwrap(),
            OpenOutfitDialog
        ));
    }
}
