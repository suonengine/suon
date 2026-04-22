use std::collections::HashMap;

use suon_database::prelude::SnapshotTable;
use suon_macros::Table;

use crate::offer::{MarketActorName, MarketOffer, MarketOfferId};

/// Table containing actor names keyed by actor id.
#[derive(Debug, Default, Table)]
pub struct MarketActorsTable {
    by_id: HashMap<u32, String>,
}

impl MarketActorsTable {
    /// Replaces the full actor-name snapshot.
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketActorName>) {
        self.by_id = rows
            .into_iter()
            .map(|row| (row.id(), row.name().to_owned()))
            .collect();
    }

    /// Inserts or replaces a single actor-name row.
    pub fn insert(&mut self, row: MarketActorName) -> Option<String> {
        self.by_id.insert(row.id(), row.name().to_owned())
    }

    /// Returns the cached actor name for the identifier.
    pub fn name(&self, actor_id: u32) -> Option<&str> {
        self.by_id.get(&actor_id).map(String::as_str)
    }

    /// Returns the number of cached actor rows.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns whether the table currently has no cached actor rows.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Returns a detached snapshot of all actor-name rows.
    pub fn rows(&self) -> Vec<MarketActorName> {
        self.by_id
            .iter()
            .map(|(id, name)| MarketActorName::new(*id, name.clone()))
            .collect()
    }
}

impl SnapshotTable for MarketActorsTable {
    type Row = MarketActorName;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketActorsTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketActorsTable::rows(self)
    }
}

/// Table containing market item names keyed by item id.
#[derive(Debug, Default, Table)]
pub struct MarketItemsTable {
    by_id: HashMap<u16, String>,
}

impl MarketItemsTable {
    /// Replaces the full item snapshot.
    pub fn replace(&mut self, rows: impl IntoIterator<Item = (u16, String)>) {
        self.by_id = rows.into_iter().collect();
    }

    /// Inserts or replaces a single item row.
    pub fn insert(&mut self, item_id: u16, name: impl Into<String>) -> Option<String> {
        self.by_id.insert(item_id, name.into())
    }

    /// Returns the cached item name for the identifier.
    pub fn name(&self, item_id: u16) -> Option<&str> {
        self.by_id.get(&item_id).map(String::as_str)
    }

    /// Returns the number of cached item rows.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns whether the table currently has no cached item rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a detached snapshot of all item rows.
    pub fn rows(&self) -> Vec<(u16, String)> {
        self.by_id
            .iter()
            .map(|(id, name)| (*id, name.clone()))
            .collect()
    }
}

impl SnapshotTable for MarketItemsTable {
    type Row = (u16, String);

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketItemsTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketItemsTable::rows(self)
    }
}

/// Table containing currently known market offers.
#[derive(Debug, Default, Table)]
pub struct MarketOffersTable {
    by_id: HashMap<MarketOfferId, MarketOffer>,
}

impl MarketOffersTable {
    /// Replaces the full offer snapshot.
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketOffer>) {
        self.by_id = rows.into_iter().map(|row| (row.id(), row)).collect();
    }

    /// Inserts or replaces a single offer row.
    pub fn insert(&mut self, row: MarketOffer) -> Option<MarketOffer> {
        self.by_id.insert(row.id(), row)
    }

    /// Inserts a newly created offer row.
    pub fn create_offer(&mut self, offer: MarketOffer) -> Option<MarketOffer> {
        self.insert(offer)
    }

    /// Returns the cached offer for the identifier.
    pub fn get(&self, id: &MarketOfferId) -> Option<&MarketOffer> {
        self.by_id.get(id)
    }

    /// Removes the cached offer for the identifier.
    pub fn remove(&mut self, id: &MarketOfferId) -> Option<MarketOffer> {
        self.by_id.remove(id)
    }

    /// Returns the number of cached offers.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns whether the table currently has no cached offers.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the cached offer rows.
    pub fn iter(&self) -> impl Iterator<Item = &MarketOffer> {
        self.by_id.values()
    }

    /// Returns a detached snapshot of all offer rows.
    pub fn rows(&self) -> Vec<MarketOffer> {
        self.by_id.values().cloned().collect()
    }
}

impl SnapshotTable for MarketOffersTable {
    type Row = MarketOffer;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketOffersTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketOffersTable::rows(self)
    }
}

impl FromIterator<MarketOffer> for MarketOffersTable {
    fn from_iter<T: IntoIterator<Item = MarketOffer>>(iter: T) -> Self {
        let mut table = Self::default();
        table.replace(iter);
        table
    }
}
