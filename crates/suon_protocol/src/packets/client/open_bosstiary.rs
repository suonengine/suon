//! Client open-bosstiary packet.

use super::prelude::*;

/// Sent by the client to open the bosstiary window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBosstiaryPacket;

impl Decodable for OpenBosstiaryPacket {
    const KIND: PacketKind = PacketKind::OpenBosstiary;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_bosstiary() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            OpenBosstiaryPacket::decode(&mut payload).unwrap(),
            OpenBosstiaryPacket
        ));
    }
}
