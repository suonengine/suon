//! Startup systems that load market settings and configure persistent tables.

use bevy::prelude::*;
use log::info;
use suon_database::prelude::*;

use crate::{
    offer::{MarketActorsTable, MarketItemsTable, MarketOfferSequence, MarketOffersTable},
    persistence::MarketSettings,
};

/// Builds the default market settings resource when the app did not provide one.
fn load_market_settings() -> MarketSettings {
    #[cfg(test)]
    {
        MarketSettings::default()
    }

    #[cfg(not(test))]
    {
        MarketSettings::load_or_default().expect("Failed to load market settings.")
    }
}

/// Inserts the market settings resource during startup when none was provided.
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

/// Configures the per-market-table persistent settings using the active
/// [`MarketSettings`] resource.
pub(super) fn configure_market_db_tables(mut commands: Commands, settings: Res<MarketSettings>) {
    let persistence = settings.persistence();
    let connection_override = persistence.database_override().cloned();
    let flush_interval = persistence.flush_interval();
    let save_on_shutdown = persistence.save_on_shutdown();

    commands.insert_resource(DbTableSettings::<MarketActorsTable>::new(
        flush_interval,
        save_on_shutdown,
        connection_override.clone(),
    ));

    commands.insert_resource(DbTableSettings::<MarketItemsTable>::new(
        flush_interval,
        save_on_shutdown,
        connection_override.clone(),
    ));

    commands.insert_resource(DbTableSettings::<MarketOffersTable>::new(
        flush_interval,
        save_on_shutdown,
        connection_override,
    ));

    info!(
        "Market persistent tables configured: interval_secs={:.3}, save_on_shutdown={}, \
         override={}",
        flush_interval.as_secs_f32().max(0.001),
        save_on_shutdown,
        persistence.database_override().is_some()
    );
}

/// Seeds runtime offer ids from the loaded persistent offers snapshot.
pub(super) fn seed_market_offer_sequence(
    offers: Db<MarketOffersTable>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
) {
    offer_sequence.seed_from_offers(&offers);
}
