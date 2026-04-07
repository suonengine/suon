//! Client channels packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelsPacket;

impl Decodable for ChannelsPacket {
    const KIND: PacketKind = PacketKind::Channels;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            ChannelsPacket::decode(&mut payload).unwrap(),
            ChannelsPacket
        ));
    }
}
