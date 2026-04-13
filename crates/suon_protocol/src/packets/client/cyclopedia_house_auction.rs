//! Client cyclopedia-house-auction packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// House-auction action requested through the cyclopedia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CyclopediaHouseAuctionAction {
    /// Requests houses for a town name.
    BrowseTown {
        /// Town name used by the query.
        town_name: String,
    },
    /// Places a bid for a house.
    PlaceBid {
        /// House identifier receiving the bid.
        house_id: u32,
        /// Bid value in gold.
        bid_value: u64,
    },
    /// Schedules a move-out for a house.
    ScheduleMoveOut {
        /// House identifier.
        house_id: u32,
        /// Timestamp associated with the move-out window.
        timestamp: u32,
    },
    /// Transfers a house to a new owner.
    Transfer {
        /// House identifier.
        house_id: u32,
        /// Transfer timestamp.
        timestamp: u32,
        /// Receiver player name.
        new_owner: String,
        /// Transfer bid value.
        bid_value: u64,
    },
    /// Cancels a scheduled move-out.
    CancelMoveOut {
        /// House identifier.
        house_id: u32,
    },
    /// Cancels an in-progress transfer.
    CancelTransfer {
        /// House identifier.
        house_id: u32,
    },
    /// Accepts a house transfer.
    AcceptTransfer {
        /// House identifier.
        house_id: u32,
    },
    /// Rejects a house transfer.
    RejectTransfer {
        /// House identifier.
        house_id: u32,
    },
}

/// Packet sent by the client to perform a cyclopedia house-auction action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CyclopediaHouseAuction {
    /// House-auction action requested by the client.
    pub action: CyclopediaHouseAuctionAction,
}

impl Decodable for CyclopediaHouseAuction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => CyclopediaHouseAuctionAction::BrowseTown {
                town_name: bytes.get_string()?,
            },
            1 => CyclopediaHouseAuctionAction::PlaceBid {
                house_id: bytes.get_u32()?,
                bid_value: bytes.get_u64()?,
            },
            2 => CyclopediaHouseAuctionAction::ScheduleMoveOut {
                house_id: bytes.get_u32()?,
                timestamp: bytes.get_u32()?,
            },
            3 => CyclopediaHouseAuctionAction::Transfer {
                house_id: bytes.get_u32()?,
                timestamp: bytes.get_u32()?,
                new_owner: bytes.get_string()?,
                bid_value: bytes.get_u64()?,
            },
            4 => CyclopediaHouseAuctionAction::CancelMoveOut {
                house_id: bytes.get_u32()?,
            },
            5 => CyclopediaHouseAuctionAction::CancelTransfer {
                house_id: bytes.get_u32()?,
            },
            6 => CyclopediaHouseAuctionAction::AcceptTransfer {
                house_id: bytes.get_u32()?,
            },
            7 => CyclopediaHouseAuctionAction::RejectTransfer {
                house_id: bytes.get_u32()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "house_action_type",
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
    fn should_decode_transfer_house_action() {
        let mut payload: &[u8] = &[
            3, 0x78, 0x56, 0x34, 0x12, 0x22, 0x11, 0x00, 0x00, 4, 0, b'J', b'o', b'a', b'o', 0x88,
            0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11,
        ];

        let packet =
            CyclopediaHouseAuction::decode(PacketKind::CyclopediaHouseAuction, &mut payload)
                .expect("CyclopediaHouseAuction packets should decode transfer actions");

        assert_eq!(
            packet.action,
            CyclopediaHouseAuctionAction::Transfer {
                house_id: 0x12345678,
                timestamp: 0x1122,
                new_owner: "Joao".to_string(),
                bid_value: 0x1122334455667788,
            }
        );
        assert!(payload.is_empty());
    }
}
