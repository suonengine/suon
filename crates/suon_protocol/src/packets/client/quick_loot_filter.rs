//! Client quick-loot-filter packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to replace the quick-loot filter list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickLootFilter {
    /// Filter mode selected by the client.
    pub filter: u8,

    /// Item ids included in the filter list.
    pub item_ids: Vec<u16>,
}

impl Decodable for QuickLootFilter {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let filter = bytes.get_u8()?;
        let count = bytes.get_u16()?;
        let mut item_ids = Vec::with_capacity(count as usize);
        for _ in 0..count {
            item_ids.push(bytes.get_u16()?);
        }

        Ok(Self { filter, item_ids })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_quick_loot_filter() {
        let mut payload: &[u8] = &[1, 2, 0, 0x34, 0x12, 0x78, 0x56];

        let packet = QuickLootFilter::decode(PacketKind::QuickLootFilter, &mut payload)
            .expect("QuickLootFilter packets should decode the filter mode and item ids");

        assert_eq!(packet.filter, 1);
        assert_eq!(packet.item_ids, vec![0x1234, 0x5678]);
        assert!(payload.is_empty());
    }
}
