use bevy::prelude::*;
use suon_protocol_client::prelude::MarketBrowseKind;

use crate::browse::MarketBrowseScope;

/// Intent requesting a market browse operation for the target actor.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketBrowseIntent {
    #[event_target]
    /// Client entity that requested the browse action.
    pub client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub actor_id: Option<u32>,
    /// Resolved browse scope requested by the caller.
    pub scope: MarketBrowseScope,
}

/// Error emitted when a market browse request cannot be translated into an intent.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketBrowseRejected {
    #[event_target]
    /// Client entity whose browse request was rejected.
    pub(crate) client: Entity,
    /// High-level browse kind carried by the rejected packet.
    pub(crate) request_kind: MarketBrowseKind,
    /// Optional item identifier carried by the rejected packet.
    pub(crate) item_id: Option<u16>,
}

impl MarketBrowseRejected {
    /// Returns the client entity whose request was rejected.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the raw high-level browse packet kind.
    pub fn request_kind(&self) -> MarketBrowseKind {
        self.request_kind
    }

    /// Returns the optional item identifier carried by the rejected request.
    pub fn item_id(&self) -> Option<u16> {
        self.item_id
    }
}

/// Triggered when a market browse operation completes successfully.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketBrowse {
    #[event_target]
    /// Client entity that completed the browse action.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Resolved browse scope for the successful operation.
    pub(crate) scope: MarketBrowseScope,
}

impl MarketBrowse {
    /// Returns the client entity that completed the browse action.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the persistent actor identifier bound to the client, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the resolved browse scope.
    pub fn scope(&self) -> &MarketBrowseScope {
        &self.scope
    }
}
