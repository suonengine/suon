use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use bevy::prelude::*;

use crate::{
    history::MarketHistoryEntry,
    offer::{MarketActorName, MarketOffer},
};

/// ORM-style abstraction used by market persistence during startup.
pub trait MarketOrm: Send + Sync + 'static {
    /// Loads actor-name reference data.
    fn load_actors(&self) -> Result<Vec<MarketActorName>>;
    /// Loads item-name reference data.
    fn load_items(&self) -> Result<Vec<(u16, String)>>;
    /// Loads active market offers.
    fn load_offers(&self) -> Result<Vec<MarketOffer>>;

    /// Persists actor-name reference data.
    fn save_actors(&self, _: &[MarketActorName]) -> Result<()> {
        Ok(())
    }

    /// Persists item-name reference data.
    fn save_items(&self, _: &[(u16, String)]) -> Result<()> {
        Ok(())
    }

    /// Persists active market offers.
    fn save_offers(&self, _: &[MarketOffer]) -> Result<()> {
        Ok(())
    }

    /// Appends a single market history entry.
    fn insert_history(&self, _: &MarketHistoryEntry) -> Result<()> {
        Ok(())
    }
}

/// Resource wrapper storing the active market ORM provider.
#[derive(Resource, Clone)]
pub struct MarketOrmResource(Arc<dyn MarketOrm>);

impl MarketOrmResource {
    /// Creates a new resource wrapper for the active market ORM provider.
    pub fn new(orm: Arc<dyn MarketOrm>) -> Self {
        Self(orm)
    }

    /// Returns the underlying market ORM provider.
    pub fn provider(&self) -> &dyn MarketOrm {
        self.0.as_ref()
    }
}

impl Deref for MarketOrmResource {
    type Target = dyn MarketOrm;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}
