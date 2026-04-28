use bevy::prelude::*;
use suon_protocol_client::prelude::MarketBrowseKind;

/// Error emitted when a market browse request cannot be translated into an intent.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct BrowseMarketRejected {
    #[event_target]
    /// Entity whose browse request was rejected.
    pub(crate) entity: Entity,

    /// High-level browse kind carried by the rejected packet.
    pub(crate) request_kind: MarketBrowseKind,
}

impl BrowseMarketRejected {
    /// Returns the entity whose request was rejected.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the raw high-level browse packet kind.
    pub fn request_kind(&self) -> MarketBrowseKind {
        self.request_kind
    }
}

/// Triggered when a market browse operation completes successfully.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct BrowseMarket {
    #[event_target]
    /// Entity that completed the browse action.
    pub(crate) entity: Entity,

    /// Resolved browse scope for the successful operation.
    pub(crate) scope: MarketBrowseKind,
}

impl BrowseMarket {
    /// Returns the entity that completed the browse action.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the resolved browse scope.
    pub fn scope(&self) -> MarketBrowseKind {
        self.scope
    }
}
