//! Client open-rule-violation packet.

use super::prelude::*;

/// Packet sent by the client to open a rule-violation conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenRuleViolation;

impl Decodable for OpenRuleViolation {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_rule_violation() {
        let mut payload: &[u8] = &[];

        let packet = OpenRuleViolation::decode(PacketKind::OpenRuleViolation, &mut payload)
            .expect("OpenRuleViolation packets should decode empty payloads");

        assert!(matches!(packet, OpenRuleViolation));
        assert!(payload.is_empty());
    }
}
