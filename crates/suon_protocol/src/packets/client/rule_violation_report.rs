//! Client rule-violation-report packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleViolationReport {
    pub report_type: u8,
    pub report_reason: u8,
    pub target_name: String,
    pub comment: String,
    pub translation: Option<String>,
    pub statement_id: Option<u32>,
}

impl Decodable for RuleViolationReport {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let report_type = bytes.get_u8()?;
        let report_reason = bytes.get_u8()?;
        let target_name = bytes.get_string()?;
        let comment = bytes.get_string()?;
        let translation = if !bytes.is_empty() {
            Some(bytes.get_string()?)
        } else {
            None
        };
        let statement_id = if bytes.len() >= 4 {
            Some(bytes.get_u32()?)
        } else {
            None
        };

        Ok(Self {
            report_type,
            report_reason,
            target_name,
            comment,
            translation,
            statement_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_rule_violation_report() {
        let mut payload: &[u8] = &[
            1, 2, 4, 0, b'N', b'a', b'm', b'e', 7, 0, b'c', b'o', b'm', b'm', b'e', b'n', b't',
        ];
        let packet =
            RuleViolationReport::decode(PacketKind::RuleViolationReport, &mut payload).unwrap();
        assert_eq!(packet.report_reason, 2);
        assert_eq!(packet.target_name, "Name");
    }
}
