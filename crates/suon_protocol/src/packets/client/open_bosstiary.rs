//! Client open-bosstiary packet.

use super::prelude::*;

/// Packet sent by the client to request bosstiary data from the server.
///
/// The opcode is payload-free and serves as the feature entry command for the
/// bosstiary flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBosstiary;

impl Decodable for OpenBosstiary {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
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
            OpenBosstiary::decode(PacketKind::OpenBosstiary, &mut payload).unwrap(),
            OpenBosstiary
        ));
    }
}
