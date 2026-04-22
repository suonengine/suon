use std::sync::Arc;

use bevy::prelude::*;
use log::{debug, info, warn};
use suon_database::prelude::*;

use crate::{
    offer::{MarketActorsTable, MarketItemsTable, MarketOfferSequence, MarketOffersTable},
    persistence::{MarketDatabaseOrm, MarketOrm, MarketOrmResource, MarketSettings},
};

use super::flush::MarketFlushTimer;

/// Builds the default market settings resource when the app did not provide one.
pub(super) fn load_market_settings() -> MarketSettings {
    #[cfg(test)]
    {
        MarketSettings::default()
    }

    #[cfg(not(test))]
    {
        MarketSettings::load_or_default().expect("Failed to load market settings.")
    }
}

/// Builds the default market ORM adapter from the active settings.
pub(super) fn build_market_orm(
    settings: &MarketSettings,
    shared_database: Option<&DatabaseSettings>,
) -> anyhow::Result<Arc<dyn MarketOrm>> {
    let database = settings
        .persistence()
        .database_override()
        .or(shared_database)
        .cloned()
        .unwrap_or_else(|| {
            DatabaseSettings::load_or_default().expect("Failed to load shared database settings.")
        });

    Ok(Arc::new(MarketDatabaseOrm::connect(&database)?))
}

/// Inserts the default market settings resource during startup when none was provided.
pub(super) fn initialize_market_settings(
    mut commands: Commands,
    settings: Option<Res<MarketSettings>>,
) {
    if let Some(settings) = settings {
        info!(
            "Market settings already provided by app: {}",
            settings.summary()
        );
        return;
    }

    let settings = load_market_settings();
    info!("Market settings loaded: {}", settings.summary());
    commands.insert_resource(settings);
}

/// Initializes the autosave timer after the market settings resource is available.
pub(super) fn initialize_market_flush_timer(
    mut commands: Commands,
    settings: Res<MarketSettings>,
    timer: Option<Res<MarketFlushTimer>>,
) {
    if timer.is_some() {
        return;
    }

    commands.insert_resource(MarketFlushTimer(Timer::from_seconds(
        settings
            .persistence()
            .flush_interval()
            .as_secs_f32()
            .max(0.001),
        TimerMode::Repeating,
    )));

    info!(
        "Market flush timer initialized: interval_secs={:.3}, save_on_shutdown={}",
        settings
            .persistence()
            .flush_interval()
            .as_secs_f32()
            .max(0.001),
        settings.persistence().save_on_shutdown()
    );
}

/// Builds the default market ORM resource during startup when the app did not provide one.
pub(super) fn initialize_market_orm(
    mut commands: Commands,
    orm: Option<Res<MarketOrmResource>>,
    settings: Res<MarketSettings>,
    shared_database: Option<Res<DatabaseSettings>>,
) {
    if orm.is_some() {
        info!("Market ORM already provided by app.");
        return;
    }

    let shared_database = shared_database.as_deref();
    let orm =
        build_market_orm(&settings, shared_database).expect("Failed to build market ORM provider");
    commands.insert_resource(MarketOrmResource::new(orm));

    if let Some(database) = settings.persistence().database_override() {
        info!(
            "Market ORM initialized from market database override: {}",
            database.summary()
        );
    } else if let Some(database) = shared_database {
        info!(
            "Market ORM initialized from shared database settings: {}",
            database.summary()
        );
    } else {
        info!("Market ORM initialized from shared database settings loaded from disk.");
    }
}

/// Loads persisted market tables into the in-memory cache during startup.
#[allow(clippy::too_many_arguments)]
pub(super) fn load_market_tables_on_startup(
    orm: Option<Res<MarketOrmResource>>,
    mut actors: DatabaseMut<MarketActorsTable>,
    mut items: DatabaseMut<MarketItemsTable>,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
) {
    let Some(orm) = orm else {
        debug!(
            "MarketPersistencePlugin started without a MarketOrmResource; market tables stay \
             empty until populated by the app"
        );
        return;
    };

    match orm.load_actors() {
        Ok(rows) => {
            let count = rows.len();
            actors.replace(rows);
            info!("Loaded {count} market actor names into MarketActorsTable");
        }
        Err(error) => warn!("Failed to load market actor names: {error:#}"),
    }

    match orm.load_items() {
        Ok(rows) => {
            let count = rows.len();
            items.replace(rows);
            info!("Loaded {count} market item names into MarketItemsTable");
        }
        Err(error) => warn!("Failed to load market item names: {error:#}"),
    }

    match orm.load_offers() {
        Ok(rows) => {
            let count = rows.len();
            offers.replace(rows);
            offer_sequence.seed_from_offers(&offers);
            info!("Loaded {count} market offers into MarketOffersTable");
        }
        Err(error) => warn!("Failed to load market offers: {error:#}"),
    }
}
