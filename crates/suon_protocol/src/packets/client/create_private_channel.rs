//! Client create-private-channel packet.

use super::prelude::*;

/// Packet sent by the client to create a private channel owned by the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatePrivateChannel;

impl Decodable for CreatePrivateChannel {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_create_private_channel() {
        let mut payload: &[u8] = &[];

        let packet = CreatePrivateChannel::decode(PacketKind::CreatePrivateChannel, &mut payload)
            .expect("CreatePrivateChannel packets should decode empty payloads");

        assert!(matches!(packet, CreatePrivateChannel));
        assert!(payload.is_empty());
    }
}
