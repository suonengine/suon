//! Market persistence wiring.
//!
//! The market crate keeps its persistence split into small modules: settings,
//! Diesel-backed table impls, and the startup wiring that bridges market
//! settings into `suon_database`'s persistent-table pipeline.

mod database;
mod settings;
mod startup;

use bevy::prelude::*;
use suon_database::prelude::*;

use crate::offer::{MarketActorsTable, MarketItemsTable, MarketOffersTable};

pub use self::{
    database::MarketHistoryJournal,
    settings::{
        MarketOfferCreateRule, MarketPersistenceSettings, MarketPolicySettings, MarketSettings,
    },
};

/// Internal plugin that wires market settings, persistent tables, and the
/// history journal into the Bevy app.
pub(crate) struct MarketPersistencePlugin;

impl Plugin for MarketPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DbPlugin)
            .add_systems(
                PreStartup,
                (
                    startup::initialize_market_settings,
                    startup::configure_market_db_tables,
                )
                    .chain(),
            )
            .init_dbpersistent::<MarketActorsTable>()
            .init_dbpersistent::<MarketItemsTable>()
            .init_dbpersistent::<MarketOffersTable>()
            .init_dbjournal::<MarketHistoryJournal>()
            .add_systems(PostStartup, startup::seed_market_offer_sequence);
    }
}
