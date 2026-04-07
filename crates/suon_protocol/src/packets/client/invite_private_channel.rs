//! Client invite-private-channel packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvitePrivateChannelPacket {
    pub receiver: String,
}

impl Decodable for InvitePrivateChannelPacket {
    const KIND: PacketKind = PacketKind::InvitePrivateChannel;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            receiver: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_invite_private_channel() {
        let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];
        assert_eq!(
            InvitePrivateChannelPacket::decode(&mut payload)
                .unwrap()
                .receiver,
            "John"
        );
    }
}
