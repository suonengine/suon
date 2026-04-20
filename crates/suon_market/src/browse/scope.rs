use suon_protocol_client::prelude::{BrowseMarketPacket, MarketBrowseKind};

/// Higher-level interpretation of the client's market browse action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketBrowseScope {
    /// Requests the caller's own active offers.
    OwnOffers,
    /// Requests the caller's own market-history view.
    OwnHistory,
    /// Requests offers for a specific item identifier.
    Item { item_id: u16 },
}

impl MarketBrowseScope {
    /// Returns the protocol browse kind that corresponds to the scope.
    pub fn request_kind(&self) -> MarketBrowseKind {
        match self {
            Self::OwnOffers => MarketBrowseKind::OwnOffers,
            Self::OwnHistory => MarketBrowseKind::OwnHistory,
            Self::Item { .. } => MarketBrowseKind::Item,
        }
    }

    /// Returns the item identifier when the browse scope targets a specific item.
    pub fn item_id(&self) -> Option<u16> {
        match self {
            Self::Item { item_id } => Some(*item_id),
            Self::OwnOffers | Self::OwnHistory => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryMarketRequestKindFromPacketError {
    /// The packet described an item browse but omitted the item identifier payload.
    MissingItemId { request_kind: MarketBrowseKind },
}

impl std::fmt::Display for TryMarketRequestKindFromPacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingItemId { request_kind } => {
                write!(
                    f,
                    "market browse request kind {:?} requires an item id",
                    request_kind
                )
            }
        }
    }
}

impl std::error::Error for TryMarketRequestKindFromPacketError {}

impl TryFrom<&BrowseMarketPacket> for MarketBrowseScope {
    type Error = TryMarketRequestKindFromPacketError;

    fn try_from(packet: &BrowseMarketPacket) -> Result<Self, Self::Error> {
        match packet.request_kind {
            MarketBrowseKind::OwnOffers => Ok(Self::OwnOffers),
            MarketBrowseKind::OwnHistory => Ok(Self::OwnHistory),
            MarketBrowseKind::Item => {
                let item_id = packet.sprite_id.ok_or(
                    TryMarketRequestKindFromPacketError::MissingItemId {
                        request_kind: MarketBrowseKind::Item,
                    },
                )?;

                Ok(Self::Item { item_id })
            }
        }
    }
}
