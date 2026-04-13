//! Client close-rule-violation packet.

use super::prelude::*;

/// Packet sent by the client to close a rule-violation conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseRuleViolation;

impl Decodable for CloseRuleViolation {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_close_rule_violation() {
        let mut payload: &[u8] = &[];

        let packet = CloseRuleViolation::decode(PacketKind::CloseRuleViolation, &mut payload)
            .expect("CloseRuleViolation packets should decode empty payloads");

        assert!(matches!(packet, CloseRuleViolation));
        assert!(payload.is_empty());
    }
}
