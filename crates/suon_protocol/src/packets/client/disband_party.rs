//! Client disband-party packet.

use super::prelude::*;

/// Sent by the party leader to forcibly disband the entire party.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisbandPartyPacket;

impl Decodable for DisbandPartyPacket {
    const KIND: PacketKind = PacketKind::DisbandParty;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            DisbandPartyPacket::decode(&mut payload).unwrap(),
            DisbandPartyPacket
        ));
    }
}
