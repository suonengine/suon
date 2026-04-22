mod database;
mod dirty;
mod flush;
mod orm;
mod settings;
mod startup;

use bevy::prelude::*;
use suon_database::prelude::*;

use crate::offer::{MarketActorsTable, MarketItemsTable, MarketOffersTable};

pub(crate) use self::{database::MarketDatabaseOrm, dirty::MarketDirty};
pub use self::{
    orm::{MarketOrm, MarketOrmResource},
    settings::{
        MarketOfferCreateRule, MarketPersistenceSettings, MarketPolicySettings, MarketSettings,
    },
};

pub(crate) struct MarketPersistencePlugin;

impl Plugin for MarketPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DatabasePlugin);
        app.init_database_table::<MarketActorsTable>()
            .init_database_table::<MarketItemsTable>()
            .init_database_table::<MarketOffersTable>();
        app.init_resource::<MarketDirty>();
        app.add_systems(
            Startup,
            (
                startup::initialize_market_settings,
                startup::initialize_market_flush_timer,
                startup::initialize_market_orm,
                startup::load_market_tables_on_startup,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                flush::autosave_market_tables,
                flush::save_market_tables_on_app_exit,
            ),
        );
    }
}
