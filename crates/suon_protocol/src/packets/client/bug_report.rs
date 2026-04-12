//! Client bug-report packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Bug category value used by the map-report flow.
const MAP_BUG_CATEGORY: u8 = 0;

/// Packet sent by the client to submit a bug report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BugReportPacket {
    /// Bug category selected by the client.
    pub category: u8,

    /// Report body written by the player.
    pub message: String,

    /// Optional map position attached to map bug reports.
    pub position: Option<Position>,
}

impl Decodable for BugReportPacket {
    const KIND: PacketKind = PacketKind::BugReport;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let category = bytes.get_u8()?;
        let message = bytes.get_string()?;
        let position = if category == MAP_BUG_CATEGORY {
            Some(bytes.get_position()?)
        } else {
            None
        };

        Ok(Self {
            category,
            message,
            position,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_map_bug_report() {
        let mut payload: &[u8] = &[0, 4, 0, b'b', b'u', b'g', b'!', 0x34, 0x12, 0x78, 0x56];

        let packet = BugReportPacket::decode(&mut payload)
            .expect("BugReport packets should decode map bug reports with a position");

        assert_eq!(packet.category, 0);
        assert_eq!(packet.message, "bug!");
        assert_eq!(
            packet.position,
            Some(Position {
                x: 0x1234,
                y: 0x5678,
            })
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_non_map_bug_report_without_position() {
        let mut payload: &[u8] = &[2, 5, 0, b'c', b'r', b'a', b's', b'h'];

        let packet = BugReportPacket::decode(&mut payload)
            .expect("BugReport packets should omit positions for non-map categories");

        assert_eq!(packet.position, None);
    }
}
