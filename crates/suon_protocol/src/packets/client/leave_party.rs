//! Client leave-party packet.

use super::prelude::*;

/// Packet sent by the client to leave the current party without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::LeaveParty};
///
/// let mut payload: &[u8] = &[];
/// let packet = LeaveParty::decode(PacketKind::LeaveParty, &mut payload).unwrap();
///
/// assert!(matches!(packet, LeaveParty));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveParty;

impl Decodable for LeaveParty {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(LeaveParty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_party_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = LeaveParty::decode(PacketKind::LeaveParty, &mut payload)
            .expect("LeaveParty packets should decode without payload bytes");

        assert!(matches!(packet, LeaveParty));
        assert!(
            payload.is_empty(),
            "LeaveParty decoding should not consume any payload bytes"
        );
    }
}
