//! Client disband-party packet.

use super::prelude::*;

/// Sent by the party leader to forcibly disband the entire party.
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
