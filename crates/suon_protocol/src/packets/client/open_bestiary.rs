//! Client open-bestiary packet.

use super::prelude::*;

/// Sent by the client to open the bestiary window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBestiaryPacket;

impl Decodable for OpenBestiaryPacket {
    const KIND: PacketKind = PacketKind::OpenBestiary;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            OpenBestiaryPacket::decode(&mut payload).unwrap(),
            OpenBestiaryPacket
        ));
    }
}
