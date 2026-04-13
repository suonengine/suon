//! Client revoke-party-invite packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to revoke a previously sent party invite.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::RevokePartyInvite};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = RevokePartyInvite::decode(PacketKind::RevokePartyInvite, &mut payload).unwrap();
///
/// assert_eq!(packet.target_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevokePartyInvite {
    /// Creature id of the player whose invite should be revoked.
    pub target_id: u32,
}

impl Decodable for RevokePartyInvite {
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
    fn should_decode_revoke_party_invite() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = RevokePartyInvite::decode(PacketKind::RevokePartyInvite, &mut payload)
            .expect("RevokePartyInvite packets should decode the target player id");

        assert_eq!(packet.target_id, 0x12345678);
        assert!(
            payload.is_empty(),
            "RevokePartyInvite decoding should consume the whole payload"
        );
    }
}
