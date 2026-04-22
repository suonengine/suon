use bevy::prelude::*;

/// Tracks whether in-memory market tables changed since the last successful flush.
#[derive(Debug, Resource, Default)]
pub(crate) struct MarketDirty(pub(crate) bool);

impl MarketDirty {
    /// Marks the market snapshot as dirty.
    pub(crate) fn mark(&mut self) {
        self.0 = true;
    }

    /// Clears the dirty marker after a successful flush.
    pub(super) fn clear(&mut self) {
        self.0 = false;
    }

    /// Returns whether there are no pending in-memory writes.
    pub(super) fn is_clean(&self) -> bool {
        !self.0
    }
}
