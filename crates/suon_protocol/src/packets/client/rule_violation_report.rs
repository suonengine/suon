//! Client rule-violation-report packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

const REPORT_TYPE_NAME: u8 = 0;
const REPORT_TYPE_STATEMENT: u8 = 1;

/// Packet sent by the client to report a rule violation, carrying the report type, reason, target name, and optional statement reference.
///
/// The optional tail depends on `report_type`: name reports append only a
/// translation string, while statement reports append both a translation string
/// and the referenced `statement_id`.
///
/// # Examples
/// ```rust
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::RuleViolationReport};
///
/// let mut payload: &[u8] = &[
///     1, 2, 4, 0, b'N', b'a', b'm', b'e', 7, 0, b'c', b'o', b'm', b'm', b'e', b'n', b't',
///     5, 0, b'q', b'u', b'o', b't', b'e', 0x78, 0x56, 0x34, 0x12,
/// ];
/// let packet = RuleViolationReport::decode(PacketKind::RuleViolationReport, &mut payload).unwrap();
///
/// assert_eq!(packet.report_type, 1);
/// assert_eq!(packet.translation.as_deref(), Some("quote"));
/// assert_eq!(packet.statement_id, Some(0x12345678));
/// assert!(payload.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleViolationReport {
    /// Primary report-category byte chosen by the client.
    pub report_type: u8,
    /// Reason byte inside the selected report category.
    pub report_reason: u8,
    /// Character or report target name referenced by the complaint.
    pub target_name: String,
    /// Free-form report body entered by the player.
    pub comment: String,
    /// Optional translated text sent by clients that include an extra translation field.
    pub translation: Option<String>,
    /// Optional statement id when the report points to a specific chat message.
    pub statement_id: Option<u32>,
}

impl Decodable for RuleViolationReport {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let report_type = bytes.get_u8()?;
        let report_reason = bytes.get_u8()?;
        let target_name = bytes.get_string()?;
        let comment = bytes.get_string()?;
        let (translation, statement_id) = match report_type {
            REPORT_TYPE_NAME => (Some(bytes.get_string()?), None),
            REPORT_TYPE_STATEMENT => (Some(bytes.get_string()?), Some(bytes.get_u32()?)),
            _ => (None, None),
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
    fn should_decode_name_rule_violation_report() {
        let mut payload: &[u8] = &[
            0, 2, 4, 0, b'N', b'a', b'm', b'e', 7, 0, b'c', b'o', b'm', b'm', b'e', b'n', b't', 5,
            0, b'a', b'l', b'i', b'a', b's',
        ];

        let packet = RuleViolationReport::decode(PacketKind::RuleViolationReport, &mut payload)
            .expect("RuleViolationReport packets should decode name reports");

        assert_eq!(packet.report_type, REPORT_TYPE_NAME);
        assert_eq!(packet.report_reason, 2);
        assert_eq!(packet.target_name, "Name");
        assert_eq!(packet.translation.as_deref(), Some("alias"));
        assert_eq!(packet.statement_id, None);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_statement_rule_violation_report() {
        let mut payload: &[u8] = &[
            1, 2, 4, 0, b'N', b'a', b'm', b'e', 7, 0, b'c', b'o', b'm', b'm', b'e', b'n', b't', 5,
            0, b'q', b'u', b'o', b't', b'e', 0x78, 0x56, 0x34, 0x12,
        ];

        let packet = RuleViolationReport::decode(PacketKind::RuleViolationReport, &mut payload)
            .expect("RuleViolationReport packets should decode statement reports");

        assert_eq!(packet.report_type, REPORT_TYPE_STATEMENT);
        assert_eq!(packet.translation.as_deref(), Some("quote"));
        assert_eq!(packet.statement_id, Some(0x12345678));
        assert!(payload.is_empty());
    }

    #[test]
    fn should_leave_translation_absent_for_other_rule_violation_types() {
        let mut payload: &[u8] = &[
            9, 2, 4, 0, b'N', b'a', b'm', b'e', 7, 0, b'c', b'o', b'm', b'm', b'e', b'n', b't',
        ];

        let packet = RuleViolationReport::decode(PacketKind::RuleViolationReport, &mut payload)
            .expect("RuleViolationReport packets should allow report types without extra fields");

        assert_eq!(packet.report_type, 9);
        assert_eq!(packet.translation, None);
        assert_eq!(packet.statement_id, None);
        assert!(payload.is_empty());
    }
}
