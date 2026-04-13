//! Client browse-store-offers packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Store browse action selected by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Store {
    /// Opens the store home view.
    Home,

    /// Opens the premium-boost entry point.
    PremiumBoost {
        /// Extra selector byte sent by the client.
        service_type: u8,
    },

    /// Opens a category and subcategory.
    Category {
        /// Top-level category name.
        category_name: String,
        /// Nested subcategory name.
        subcategory_name: String,
        /// Sort order requested by the client.
        sort_order: u8,
        /// Service type requested by the client.
        service_type: u8,
    },

    /// Opens the useful-things group by list identifier.
    UsefulThings {
        /// Useful-things list id.
        offer_list_id: u8,
    },

    /// Opens a store offer by id.
    Offer {
        /// Store offer id.
        offer_id: u32,
        /// Sort order requested by the client.
        sort_order: u8,
        /// Service type requested by the client.
        service_type: u8,
    },

    /// Searches store offers by query text.
    Search {
        /// Search text sent by the client.
        query: String,
        /// Sort order requested by the client.
        sort_order: u8,
        /// Service type requested by the client.
        service_type: u8,
    },
}

/// Packet sent by the client to browse or search store offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseStoreOffers {
    /// Store browse action selected by the client.
    pub action: Store,
}

impl Decodable for BrowseStoreOffers {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => Store::Home,
            1 => Store::PremiumBoost {
                service_type: bytes.get_u8()?,
            },
            2 => Store::Category {
                category_name: bytes.get_string()?,
                subcategory_name: bytes.get_string()?,
                sort_order: bytes.get_u8()?,
                service_type: bytes.get_u8()?,
            },
            3 => Store::UsefulThings {
                offer_list_id: bytes.get_u8()?,
            },
            4 => Store::Offer {
                offer_id: bytes.get_u32()?,
                sort_order: bytes.get_u8()?,
                service_type: bytes.get_u8()?,
            },
            5 => Store::Search {
                query: bytes.get_string()?,
                sort_order: bytes.get_u8()?,
                service_type: bytes.get_u8()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
                    value,
                });
            }
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_store_category_browse() {
        let mut payload: &[u8] = &[
            2, 4, 0, b'H', b'o', b'm', b'e', 7, 0, b'O', b'u', b't', b'f', b'i', b't', b's', 3, 1,
        ];

        let packet = BrowseStoreOffers::decode(PacketKind::BrowseStoreOffers, &mut payload)
            .expect("BrowseStoreOffers packets should decode category requests");

        assert_eq!(
            packet.action,
            Store::Category {
                category_name: "Home".into(),
                subcategory_name: "Outfits".into(),
                sort_order: 3,
                service_type: 1,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_store_search_browse() {
        let mut payload: &[u8] = &[5, 4, 0, b'b', b'o', b'o', b't', 2, 0];

        let packet = BrowseStoreOffers::decode(PacketKind::BrowseStoreOffers, &mut payload)
            .expect("BrowseStoreOffers packets should decode search requests");

        assert_eq!(
            packet.action,
            Store::Search {
                query: "boot".into(),
                sort_order: 2,
                service_type: 0,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_store_offer_browse() {
        let mut payload: &[u8] = &[4, 0x78, 0x56, 0x34, 0x12, 3, 1];

        let packet = BrowseStoreOffers::decode(PacketKind::BrowseStoreOffers, &mut payload)
            .expect("BrowseStoreOffers packets should decode direct offer requests");

        assert_eq!(
            packet.action,
            Store::Offer {
                offer_id: 0x12345678,
                sort_order: 3,
                service_type: 1,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_reject_unknown_store_browse_action() {
        let mut payload: &[u8] = &[9];

        let error = BrowseStoreOffers::decode(PacketKind::BrowseStoreOffers, &mut payload)
            .expect_err("BrowseStoreOffers packets should reject unsupported actions");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "action",
                value: 9,
            }
        ));
    }
}
