//! Client invite-to-private-channel packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteToPrivateChannelPacket {
    pub name: String,
}

impl Decodable for InviteToPrivateChannelPacket {
    const KIND: PacketKind = PacketKind::InviteToPrivateChannel;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            name: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_invite_to_private_channel() {
        let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];
        assert_eq!(
            InviteToPrivateChannelPacket::decode(&mut payload)
                .unwrap()
                .name,
            "John"
        );
    }
}
