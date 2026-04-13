//! Client get-reward-daily packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// One reward item selected by the client for daily reward collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyRewardItem {
    /// Item id selected by the client.
    pub item_id: u16,

    /// Item count selected for this entry.
    pub count: u8,
}

/// Packet sent by the client to claim a daily reward selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRewardDaily {
    /// Selected bonus-shrine option.
    pub bonus_shrine: u8,

    /// Reward items selected by the client.
    pub items: Vec<DailyRewardItem>,
}

impl Decodable for GetRewardDaily {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let bonus_shrine = bytes.get_u8()?;
        let item_count = usize::from(bytes.get_u8()?);
        let mut items = Vec::with_capacity(item_count);

        for _ in 0..item_count {
            items.push(DailyRewardItem {
                item_id: bytes.get_u16()?,
                count: bytes.get_u8()?,
            });
        }

        Ok(Self {
            bonus_shrine,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_get_reward_daily() {
        let mut payload: &[u8] = &[2, 2, 0x34, 0x12, 5, 0x78, 0x56, 1];

        let packet = GetRewardDaily::decode(PacketKind::GetRewardDaily, &mut payload)
            .expect("GetRewardDaily packets should decode shrine and item selections");

        assert_eq!(packet.bonus_shrine, 2);
        assert_eq!(
            packet.items,
            vec![
                DailyRewardItem {
                    item_id: 0x1234,
                    count: 5,
                },
                DailyRewardItem {
                    item_id: 0x5678,
                    count: 1,
                },
            ]
        );
        assert!(payload.is_empty());
    }
}
