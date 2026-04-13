//! Client channels packet.

use super::prelude::*;

/// Packet sent by the client to request the list of available chat channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channels;

impl Decodable for Channels {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_channels() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            Channels::decode(PacketKind::Channels, &mut payload).unwrap(),
            Channels
        ));
    }
}
