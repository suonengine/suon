//! Client pass-party-leadership packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to pass party leadership to another player.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = PassPartyLeadershipPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.target_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PassPartyLeadershipPacket {
    /// Creature id of the player who should become the new party leader.
    pub target_id: u32,
}

impl Decodable for PassPartyLeadershipPacket {
    const KIND: PacketKind = PacketKind::PassPartyLeadership;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            target_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_pass_party_leadership() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = PassPartyLeadershipPacket::decode(&mut payload)
            .expect("PassPartyLeadership packets should decode the target player id");

        assert_eq!(packet.target_id, 0x12345678);
        assert!(
            payload.is_empty(),
            "PassPartyLeadership decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_pass_party_leadership_kind_constant() {
        assert_eq!(
            PassPartyLeadershipPacket::KIND,
            PacketKind::PassPartyLeadership,
            "PassPartyLeadership packets should advertise the correct packet kind"
        );
    }
}
