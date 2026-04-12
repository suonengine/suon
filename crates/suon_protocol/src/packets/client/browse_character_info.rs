//! Client browse-character-info packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Cyclopedia character-information section selected by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterInfoKind {
    /// Base information.
    BaseInformation = 0,
    /// General stats.
    GeneralStats = 1,
    /// Combat stats.
    CombatStats = 2,
    /// Recent deaths.
    RecentDeaths = 3,
    /// Recent PvP kills.
    RecentPvpKills = 4,
    /// Achievements.
    Achievements = 5,
    /// Item summary.
    ItemSummary = 6,
    /// Outfits and mounts.
    OutfitsAndMounts = 7,
    /// Store summary.
    StoreSummary = 8,
    /// Inspection.
    Inspection = 9,
    /// Badges.
    Badges = 10,
    /// Titles.
    Titles = 11,
    /// Wheel information.
    Wheel = 12,
    /// Offence stats.
    OffenceStats = 13,
    /// Defence stats.
    DefenceStats = 14,
    /// Miscellaneous stats.
    MiscStats = 15,
}

impl TryFrom<u8> for CharacterInfoKind {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BaseInformation),
            1 => Ok(Self::GeneralStats),
            2 => Ok(Self::CombatStats),
            3 => Ok(Self::RecentDeaths),
            4 => Ok(Self::RecentPvpKills),
            5 => Ok(Self::Achievements),
            6 => Ok(Self::ItemSummary),
            7 => Ok(Self::OutfitsAndMounts),
            8 => Ok(Self::StoreSummary),
            9 => Ok(Self::Inspection),
            10 => Ok(Self::Badges),
            11 => Ok(Self::Titles),
            12 => Ok(Self::Wheel),
            13 => Ok(Self::OffenceStats),
            14 => Ok(Self::DefenceStats),
            15 => Ok(Self::MiscStats),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "character_info_type",
                value,
            }),
        }
    }
}

/// Packet sent by the client to browse cyclopedia character information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseCharacterInfoPacket {
    /// Character id selected by the client.
    pub character_id: u32,

    /// Character-information section being browsed.
    pub info_kind: CharacterInfoKind,

    /// Entries per page for paginated sections.
    pub entries_per_page: Option<u16>,

    /// Page number for paginated sections.
    pub page: Option<u16>,
}

impl Decodable for BrowseCharacterInfoPacket {
    const KIND: PacketKind = PacketKind::BrowseCharacterInfo;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let character_id = bytes.get_u32()?;
        let info_kind = CharacterInfoKind::try_from(bytes.get_u8()?)?;
        let (entries_per_page, page) = match info_kind {
            CharacterInfoKind::RecentDeaths | CharacterInfoKind::RecentPvpKills => {
                (Some(bytes.get_u16()?), Some(bytes.get_u16()?))
            }
            _ => (None, None),
        };

        Ok(Self {
            character_id,
            info_kind,
            entries_per_page,
            page,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_paginated_character_info_browse() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 3, 10, 0, 2, 0];

        let packet = BrowseCharacterInfoPacket::decode(&mut payload)
            .expect("BrowseCharacterInfo packets should decode paginated sections");

        assert_eq!(packet.character_id, 0x12345678);
        assert_eq!(packet.info_kind, CharacterInfoKind::RecentDeaths);
        assert_eq!(packet.entries_per_page, Some(10));
        assert_eq!(packet.page, Some(2));
        assert!(payload.is_empty());
    }
}
