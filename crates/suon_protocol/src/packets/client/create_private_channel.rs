//! Client create-private-channel packet.

use super::prelude::*;

/// Packet sent by the client to create a private channel owned by the player.
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

        let packet = CreatePrivateChannelPacket::decode(&mut payload)
            .expect("CreatePrivateChannel packets should decode empty payloads");

        assert!(matches!(packet, CreatePrivateChannelPacket));
        assert!(payload.is_empty());
    }

    #[test]
    fn should_expose_create_private_channel_kind_constant() {
        assert_eq!(
            CreatePrivateChannelPacket::KIND,
            PacketKind::CreatePrivateChannel
        );
    }
}
