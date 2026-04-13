//! Client search-bestiary packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Search mode requested for the bestiary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BestiarySearchKind {
    /// Searches by a set of race ids collected from the kill tracker.
    ByRaceIds {
        /// Race ids included in the query.
        race_ids: Vec<u16>,
    },

    /// Searches by a free-form race name.
    ByName {
        /// Free-form race name.
        name: String,
    },
}

/// Packet sent by the client to search the bestiary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBestiary {
    /// Search mode and payload requested by the client.
    pub search: BestiarySearchKind,
}

impl Decodable for SearchBestiary {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let search = match bytes.get_u8()? {
            1 => {
                let amount = bytes.get_u16()?;
                let mut race_ids = Vec::with_capacity(amount as usize);
                for _ in 0..amount {
                    race_ids.push(bytes.get_u16()?);
                }

                BestiarySearchKind::ByRaceIds { race_ids }
            }
            0 => BestiarySearchKind::ByName {
                name: bytes.get_string()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "search_mode",
                    value,
                });
            }
        };

        Ok(Self { search })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_bestiary_search_by_race_ids() {
        let mut payload: &[u8] = &[1, 2, 0, 0x34, 0x12, 0x78, 0x56];

        let packet = SearchBestiary::decode(PacketKind::SearchBestiary, &mut payload)
            .expect("SearchBestiary packets should decode race-id searches");

        assert_eq!(
            packet.search,
            BestiarySearchKind::ByRaceIds {
                race_ids: vec![0x1234, 0x5678],
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_bestiary_search_by_name() {
        let mut payload: &[u8] = &[0, 4, 0, b'd', b'e', b'e', b'r'];

        let packet = SearchBestiary::decode(PacketKind::SearchBestiary, &mut payload)
            .expect("SearchBestiary packets should decode name searches");

        assert_eq!(
            packet.search,
            BestiarySearchKind::ByName {
                name: "deer".to_string(),
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_reject_unknown_bestiary_search_mode() {
        let mut payload: &[u8] = &[9];

        let error = SearchBestiary::decode(PacketKind::SearchBestiary, &mut payload)
            .expect_err("SearchBestiary packets should reject unsupported search modes");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "search_mode",
                value: 9,
            }
        ));
    }

    #[test]
    fn should_fail_when_bestiary_race_list_is_incomplete() {
        let mut payload: &[u8] = &[1, 2, 0, 0x34, 0x12];

        let error = SearchBestiary::decode(PacketKind::SearchBestiary, &mut payload).expect_err(
            "SearchBestiary packets should fail when a declared race list is truncated",
        );

        assert!(matches!(
            error,
            DecodableError::Decoder(crate::packets::decoder::DecoderError::Incomplete { .. })
        ));
    }
}
