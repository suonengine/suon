//! Client disband-party packet.

use super::prelude::*;

/// Packet sent by the client to dissolve the current party immediately.
///
/// This opcode carries no payload bytes on the wire. Its presence alone tells
/// the server that the acting player wants to disband the whole party instead
/// of merely leaving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisbandParty;

impl Decodable for DisbandParty {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_disband_party() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            DisbandParty::decode(PacketKind::DisbandParty, &mut payload).unwrap(),
            DisbandParty
        ));
    }
}
