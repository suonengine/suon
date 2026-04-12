//! Client query-boss-slot-info packet.

use super::prelude::*;

/// Sent by the client to query boss slot information from the bosstiary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueryBossSlotInfoPacket;

impl Decodable for QueryBossSlotInfoPacket {
    const KIND: PacketKind = PacketKind::QueryBossSlotInfo;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            QueryBossSlotInfoPacket::decode(&mut payload).unwrap(),
            QueryBossSlotInfoPacket
        ));
    }
}
