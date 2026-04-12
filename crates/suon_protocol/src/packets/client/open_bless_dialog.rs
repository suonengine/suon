//! Client open-bless-dialog packet.

use super::prelude::*;

/// Sent by the client to open the bless selection flow at a shrine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBlessDialogPacket;

impl Decodable for OpenBlessDialogPacket {
    const KIND: PacketKind = PacketKind::OpenBlessDialog;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            OpenBlessDialogPacket::decode(&mut payload).unwrap(),
            OpenBlessDialogPacket
        ));
    }
}
