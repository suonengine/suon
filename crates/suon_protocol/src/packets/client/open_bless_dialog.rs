//! Client open-bless-dialog packet.

use super::prelude::*;

/// Sent by the client to open the bless selection flow at a shrine.
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
        assert!(matches!(
            OpenBlessDialog::decode(PacketKind::OpenBlessDialog, &mut payload).unwrap(),
            OpenBlessDialog
        ));
    }
}
