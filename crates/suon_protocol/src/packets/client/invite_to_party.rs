//! Client invite-to-party packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to invite a target player to a party.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::InviteToParty};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = InviteToParty::decode(PacketKind::InviteToParty, &mut payload).unwrap();
///
/// assert_eq!(packet.target_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InviteToParty {
    /// Creature id of the player being invited.
    pub target_id: u32,
}

impl Decodable for InviteToParty {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            target_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_invite_to_party() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = InviteToParty::decode(PacketKind::InviteToParty, &mut payload)
            .expect("InviteToParty packets should decode the target player id");

        assert_eq!(packet.target_id, 0x12345678);
        assert!(
            payload.is_empty(),
            "InviteToParty decoding should consume the whole payload"
        );
    }
}
