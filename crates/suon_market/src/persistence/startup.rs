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
pub(super) fn build_market_orm(settings: &MarketSettings) -> anyhow::Result<Arc<dyn MarketOrm>> {
    Ok(Arc::new(MarketDatabaseOrm::connect(
        settings.persistence().database(),
    )?))
}

/// Inserts the default market settings resource during startup when none was provided.
pub(super) fn initialize_market_settings(
    mut commands: Commands,
    settings: Option<Res<MarketSettings>>,
) {
    if settings.is_some() {
        return;
    }

    commands.insert_resource(load_market_settings());
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
        settings.persistence().flush_interval_secs().max(0.001) as f32,
        TimerMode::Repeating,
    )));
}

/// Builds the default market ORM resource during startup when the app did not provide one.
pub(super) fn initialize_market_orm(
    mut commands: Commands,
    orm: Option<Res<MarketOrmResource>>,
    settings: Res<MarketSettings>,
) {
    if orm.is_some() {
        return;
    }

    let orm = build_market_orm(&settings).expect("Failed to build market ORM provider");
    commands.insert_resource(MarketOrmResource::new(orm));
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
