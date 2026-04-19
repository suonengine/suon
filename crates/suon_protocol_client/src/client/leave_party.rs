//! Client leave-party packet.

use super::prelude::*;

/// Packet sent by the client to leave the current party without payload data.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[];
/// let packet = LeavePartyPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, LeavePartyPacket));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeavePartyPacket;

impl Decodable for LeavePartyPacket {
    const KIND: PacketKind = PacketKind::LeaveParty;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(LeavePartyPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_party_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = LeavePartyPacket::decode(&mut payload)
            .expect("LeaveParty packets should decode without payload bytes");

        assert!(matches!(packet, LeavePartyPacket));
        assert!(
            payload.is_empty(),
            "LeaveParty decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_leave_party_kind_constant() {
        assert_eq!(
            LeavePartyPacket::KIND,
            PacketKind::LeaveParty,
            "LeaveParty packets should advertise the correct packet kind"
        );
    }
}
