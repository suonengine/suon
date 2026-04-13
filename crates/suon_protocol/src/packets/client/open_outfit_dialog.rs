//! Client open-outfit-dialog packet.

use super::prelude::*;

/// Packet sent by the client to request the outfit selection data set.
///
/// This packet does not carry parameters. The server uses the opcode to return
/// the outfit list, mounts, and any contextual customization data.
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
