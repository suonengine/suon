//! Client join-party packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to join the party of a target player.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::JoinParty};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = JoinParty::decode(PacketKind::JoinParty, &mut payload).unwrap();
///
/// assert_eq!(packet.target_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinParty {
    /// Creature id of the party member whose invite is being accepted.
    pub target_id: u32,
}

impl Decodable for JoinParty {
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
    fn should_decode_join_party() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = JoinParty::decode(PacketKind::JoinParty, &mut payload)
            .expect("JoinParty packets should decode the target player id");

        assert_eq!(packet.target_id, 0x12345678);
        assert!(
            payload.is_empty(),
            "JoinParty decoding should consume the whole payload"
        );
    }
}
