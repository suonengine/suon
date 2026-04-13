//! Client query-boss-slot-info packet.

use super::prelude::*;

/// Packet sent by the client to request details about a bosstiary boss slot.
///
/// The command has no extra payload in the current layout. Its semantic meaning
/// comes entirely from the opcode chosen by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryBossSlotInfo;

impl Decodable for QueryBossSlotInfo {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_query_boss_slot_info() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            QueryBossSlotInfo::decode(PacketKind::QueryBossSlotInfo, &mut payload).unwrap(),
            QueryBossSlotInfo
        ));
    }
}
