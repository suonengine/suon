use std::time::SystemTime;

use bevy::prelude::*;
use suon_database::prelude::SnapshotTable;
use suon_macros::Table;

use crate::{
    history::MarketHistoryAction,
    offer::{MarketOfferId, MarketTradeSide},
};

/// Immutable market history entry stored as an audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketHistoryEntry {
    id: u64,
    recorded_at: SystemTime,
    action: MarketHistoryAction,
    actor_id: Option<u32>,
    offer_actor_id: Option<u32>,
    item_id: Option<u16>,
    offer_id: Option<MarketOfferId>,
    amount: u16,
    remaining_amount: Option<u16>,
    price: Option<u64>,
    side: Option<MarketTradeSide>,
}

impl MarketHistoryEntry {
    /// Creates a new immutable market-history entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        recorded_at: SystemTime,
        action: MarketHistoryAction,
        actor_id: Option<u32>,
        offer_actor_id: Option<u32>,
        item_id: Option<u16>,
        offer_id: Option<MarketOfferId>,
        amount: u16,
        remaining_amount: Option<u16>,
        price: Option<u64>,
        side: Option<MarketTradeSide>,
    ) -> Self {
        Self {
            id,
            recorded_at,
            action,
            actor_id,
            offer_actor_id,
            item_id,
            offer_id,
            amount,
            remaining_amount,
            price,
            side,
        }
    }

    /// Returns the unique history-entry identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns when the history entry was recorded.
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Returns the recorded market action.
    pub fn action(&self) -> MarketHistoryAction {
        self.action
    }

    /// Returns the actor that initiated the action, when known.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the actor that owned the offer, when known.
    pub fn offer_actor_id(&self) -> Option<u32> {
        self.offer_actor_id
    }

    /// Returns the related item identifier, when known.
    pub fn item_id(&self) -> Option<u16> {
        self.item_id
    }

    /// Returns the related offer identifier, when known.
    pub fn offer_id(&self) -> Option<MarketOfferId> {
        self.offer_id
    }

    /// Returns the amount recorded in the history entry.
    pub fn amount(&self) -> u16 {
        self.amount
    }

    /// Returns the remaining amount after the action, when relevant.
    pub fn remaining_amount(&self) -> Option<u16> {
        self.remaining_amount
    }

    /// Returns the recorded price, when relevant.
    pub fn price(&self) -> Option<u64> {
        self.price
    }

    /// Returns the recorded trade side, when relevant.
    pub fn side(&self) -> Option<MarketTradeSide> {
        self.side
    }
}

/// Append-only table containing market history entries.
#[derive(Debug, Default, Table)]
pub struct MarketHistoryTable {
    entries: Vec<MarketHistoryEntry>,
}

impl MarketHistoryTable {
    /// Replaces the full history snapshot.
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketHistoryEntry>) {
        self.entries = rows.into_iter().collect();
    }

    /// Appends a new history entry to the audit log.
    pub fn append(&mut self, row: MarketHistoryEntry) {
        self.entries.push(row);
    }

    /// Returns an iterator over the cached history entries.
    pub fn iter(&self) -> impl Iterator<Item = &MarketHistoryEntry> {
        self.entries.iter()
    }

    /// Returns the number of cached history entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether the history table has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns a detached snapshot of all history entries.
    pub fn rows(&self) -> Vec<MarketHistoryEntry> {
        self.entries.clone()
    }
}

impl SnapshotTable for MarketHistoryTable {
    type Row = MarketHistoryEntry;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketHistoryTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketHistoryTable::rows(self)
    }
}

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketHistorySequence {
    next_id: u64,
}

impl MarketHistorySequence {
    pub(crate) fn seed_from_history(&mut self, history: &MarketHistoryTable) {
        self.next_id = history.iter().map(MarketHistoryEntry::id).max().unwrap_or(0);
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.next_id = self.next_id.saturating_add(1);
        self.next_id
    }
}
