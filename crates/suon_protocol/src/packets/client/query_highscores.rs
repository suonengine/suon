//! Client query-highscores packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Highscore query kind sent by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighscoreQueryKind {
    /// Queries highscore entries for a page.
    GetEntries = 0,
    /// Queries only the player's current rank.
    OurRank = 1,
}

impl TryFrom<u8> for HighscoreQueryKind {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::GetEntries),
            1 => Ok(Self::OurRank),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "request_type",
                value,
            }),
        }
    }
}

/// Packet sent by the client to query highscores data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryHighscores {
    /// Highscore query kind.
    pub query_kind: HighscoreQueryKind,

    /// Selected highscore category id.
    pub category: u8,

    /// Selected vocation filter.
    pub vocation_id: u32,

    /// Selected world name.
    pub world_name: String,

    /// Selected world category value.
    pub world_category: u8,

    /// Selected BattlEye world-type value.
    pub battleye_world_type: u8,

    /// Optional page selected for entry queries.
    pub page: Option<u16>,

    /// Number of entries selected per page.
    pub entries_per_page: u8,
}

impl Decodable for QueryHighscores {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let query_kind = HighscoreQueryKind::try_from(bytes.get_u8()?)?;
        let category = bytes.get_u8()?;
        let vocation_id = bytes.get_u32()?;
        let world_name = bytes.get_string()?;
        let world_category = bytes.get_u8()?;
        let battleye_world_type = bytes.get_u8()?;
        let page = match query_kind {
            HighscoreQueryKind::GetEntries => Some(bytes.get_u16()?),
            HighscoreQueryKind::OurRank => None,
        };
        let entries_per_page = bytes.get_u8()?;

        Ok(Self {
            query_kind,
            category,
            vocation_id,
            world_name,
            world_category,
            battleye_world_type,
            page,
            entries_per_page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_highscore_entry_query() {
        let mut payload: &[u8] = &[
            0, 8, 0x78, 0x56, 0x34, 0x12, 4, 0, b'T', b'e', b's', b't', 0, 1, 3, 0, 25,
        ];

        let packet = QueryHighscores::decode(PacketKind::QueryHighscores, &mut payload)
            .expect("QueryHighscores packets should decode entry queries");

        assert_eq!(packet.query_kind, HighscoreQueryKind::GetEntries);
        assert_eq!(packet.category, 8);
        assert_eq!(packet.vocation_id, 0x12345678);
        assert_eq!(packet.world_name, "Test");
        assert_eq!(packet.page, Some(3));
        assert_eq!(packet.entries_per_page, 25);
        assert!(payload.is_empty());
    }
}
