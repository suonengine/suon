//! Client open-bestiary packet.

use super::prelude::*;

/// Packet sent by the client to request the bestiary entrypoint state.
///
/// No additional payload is sent. The opcode itself is the trigger for the
/// server to answer with the data needed to populate the feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBestiary;

impl Decodable for OpenBestiary {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_bestiary() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenBestiary::decode(PacketKind::OpenBestiary, &mut payload).unwrap(),
            OpenBestiary
        ));
    }
}
