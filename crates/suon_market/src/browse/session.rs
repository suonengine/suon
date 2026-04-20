use bevy::prelude::*;

use crate::browse::MarketBrowseScope;

/// Tracks the current market UI state for a client entity.
#[derive(Debug, Clone, Component, Default, PartialEq, Eq)]
pub struct MarketSession {
    last_browse: Option<MarketBrowseScope>,
}

impl MarketSession {
    /// Creates a new market session snapshot.
    pub fn new(last_browse: Option<MarketBrowseScope>) -> Self {
        Self { last_browse }
    }

    /// Returns the last resolved browse scope for the open market session.
    pub fn last_browse(&self) -> Option<&MarketBrowseScope> {
        self.last_browse.as_ref()
    }
}
