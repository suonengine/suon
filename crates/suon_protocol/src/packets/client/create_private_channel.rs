//! Client create-private-channel packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatePrivateChannelPacket;

impl Decodable for CreatePrivateChannelPacket {
    const KIND: PacketKind = PacketKind::CreatePrivateChannel;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_create_private_channel() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            CreatePrivateChannelPacket::decode(&mut payload).unwrap(),
            CreatePrivateChannelPacket
        ));
    }
}
