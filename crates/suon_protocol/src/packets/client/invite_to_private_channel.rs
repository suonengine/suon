//! Client invite-to-private-channel packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InviteToPrivateChannel {
    pub name: String,
}

impl Decodable for InviteToPrivateChannel {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
            InviteToPrivateChannel::decode(PacketKind::InviteToPrivateChannel, &mut payload)
                .unwrap()
                .name,
            "John"
        );
    }
}
