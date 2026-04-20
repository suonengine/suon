use std::{ops::Deref, sync::Arc};

use anyhow::Result;
use bevy::prelude::*;

use crate::{
    history::MarketHistoryEntry,
    offer::{MarketItem, MarketOffer, PlayerName},
};

/// ORM-style abstraction used by market persistence during startup.
pub trait MarketOrm: Send + Sync + 'static {
    fn load_players(&self) -> Result<Vec<PlayerName>>;
    fn load_items(&self) -> Result<Vec<MarketItem>>;
    fn load_offers(&self) -> Result<Vec<MarketOffer>>;
    fn load_history(&self) -> Result<Vec<MarketHistoryEntry>> {
        Ok(Vec::new())
    }

    fn save_players(&self, _: &[PlayerName]) -> Result<()> {
        Ok(())
    }

    fn save_items(&self, _: &[MarketItem]) -> Result<()> {
        Ok(())
    }

    fn save_offers(&self, _: &[MarketOffer]) -> Result<()> {
        Ok(())
    }

    fn save_history(&self, _: &[MarketHistoryEntry]) -> Result<()> {
        Ok(())
    }
}

/// Resource wrapper storing the active market ORM provider.
#[derive(Resource, Clone)]
pub struct MarketOrmResource(Arc<dyn MarketOrm>);

impl MarketOrmResource {
    pub fn new(orm: Arc<dyn MarketOrm>) -> Self {
        Self(orm)
    }

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
