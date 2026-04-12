//! Client cancel-rule-violation packet.

use super::prelude::*;

/// Sent by the client to cancel the current rule violation report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelRuleViolationPacket;

impl Decodable for CancelRuleViolationPacket {
    const KIND: PacketKind = PacketKind::CancelRuleViolation;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_cancel_rule_violation() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            CancelRuleViolationPacket::decode(&mut payload).unwrap(),
            CancelRuleViolationPacket
        ));
    }
}
