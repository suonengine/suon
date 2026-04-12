//! Client invite-private-channel packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to open a private conversation with a receiver.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::InvitePrivateChannelPacket};
///
/// let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];
/// let packet = InvitePrivateChannelPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.receiver, "John");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvitePrivateChannelPacket {
    /// Name of the player receiving the private conversation invite.
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

        let packet = InvitePrivateChannelPacket::decode(&mut payload)
            .expect("InvitePrivateChannel packets should decode the receiver name");

        assert_eq!(packet.receiver, "John");
        assert!(payload.is_empty());
    }

    #[test]
    fn should_expose_invite_private_channel_kind_constant() {
        assert_eq!(
            InvitePrivateChannelPacket::KIND,
            PacketKind::InvitePrivateChannel
        );
    }
}
