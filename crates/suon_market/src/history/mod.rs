use bevy::prelude::*;
use std::time::SystemTime;
use suon_database::prelude::SnapshotTable;
use suon_macros::Table;

use crate::offer::{MarketOfferId, MarketTradeSide};

/// Action stored in the market history log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketHistoryAction {
    Create,
    Cancel,
    Accept,
}

impl std::fmt::Display for MarketHistoryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Create => "create",
            Self::Cancel => "cancel",
            Self::Accept => "accept",
        };

        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMarketHistoryActionError {
    pub value: String,
}

impl std::fmt::Display for ParseMarketHistoryActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported market history action '{}'", self.value)
    }
}

impl std::error::Error for ParseMarketHistoryActionError {}

impl std::str::FromStr for MarketHistoryAction {
    type Err = ParseMarketHistoryActionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create" => Ok(Self::Create),
            "cancel" => Ok(Self::Cancel),
            "accept" => Ok(Self::Accept),
            other => Err(ParseMarketHistoryActionError {
                value: other.to_string(),
            }),
        }
    }
}

/// Immutable market history entry stored as an audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketHistoryEntry {
    pub id: u64,
    pub recorded_at: SystemTime,
    pub action: MarketHistoryAction,
    pub actor_player_id: Option<u32>,
    pub offer_player_id: Option<u32>,
    pub item_id: Option<u16>,
    pub offer_id: Option<MarketOfferId>,
    pub amount: u16,
    pub remaining_amount: Option<u16>,
    pub price: Option<u64>,
    pub side: Option<MarketTradeSide>,
}

/// Append-only table containing market history entries.
#[derive(Debug, Default, Table)]
pub struct MarketHistoryTable {
    entries: Vec<MarketHistoryEntry>,
}

impl MarketHistoryTable {
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketHistoryEntry>) {
        self.entries = rows.into_iter().collect();
    }

    pub fn append(&mut self, row: MarketHistoryEntry) {
        self.entries.push(row);
    }

    pub fn iter(&self) -> impl Iterator<Item = &MarketHistoryEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

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
        self.next_id = history.iter().map(|entry| entry.id).max().unwrap_or(0);
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.next_id = self.next_id.saturating_add(1);
        self.next_id
    }
}

pub struct MarketHistoryPlugin;

impl Plugin for MarketHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketHistorySequence>();
    }
}
