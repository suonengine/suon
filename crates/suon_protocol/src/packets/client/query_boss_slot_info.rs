//! Client query-boss-slot-info packet.

use super::prelude::*;

/// Sent by the client to query boss slot information from the bosstiary.
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
