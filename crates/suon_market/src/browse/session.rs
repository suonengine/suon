use bevy::prelude::*;
use suon_protocol_client::prelude::MarketBrowseKind;

use crate::browse::{BrowseMarket, BrowseMarketIntent};

/// Tracks the current market UI state for a client entity.
#[derive(Debug, Clone, Component, Default, PartialEq, Eq)]
pub struct MarketSession {
    last_browse: Option<MarketBrowseKind>,
}

impl MarketSession {
    /// Creates a new market session snapshot.
    pub fn new(last_browse: Option<MarketBrowseKind>) -> Self {
        Self { last_browse }
    }

    /// Returns the last resolved browse scope for the open market session.
    pub fn last_browse(&self) -> Option<MarketBrowseKind> {
        self.last_browse
    }
}

/// Opens or refreshes the market session for a successful browse intent.
pub(super) fn on_browse_market_intent(event: On<BrowseMarketIntent>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert(MarketSession::new(Some(event.scope)));

    commands.trigger(BrowseMarket {
        entity: event.entity,
        scope: event.scope,
    });
}
