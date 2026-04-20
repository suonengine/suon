mod orm;
mod settings;
mod sqlx;

use bevy::prelude::*;
use log::{debug, info, warn};
use suon_database::prelude::*;

use crate::{
    history::{MarketHistorySequence, MarketHistoryTable},
    offer::{MarketItemsTable, MarketOfferSequence, MarketOffersTable, MarketPlayersTable},
};

pub use orm::{MarketOrm, MarketOrmResource};
pub use settings::{MarketPersistenceSettings, MarketPolicySettings, MarketSettings};
pub(crate) use sqlx::SqlxMarketOrm;

#[derive(Debug, Clone, Copy, Default, Event)]
pub struct SaveMarketData;

#[derive(Debug, Clone, Copy, Default, Event)]
pub struct ShutdownMarketData;

#[derive(Debug, Resource)]
struct MarketFlushTimer(Timer);

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketDirty(pub(crate) bool);

pub struct MarketPersistencePlugin;

impl Plugin for MarketPersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DatabasePlugin);

        let settings = if app.world().contains_resource::<MarketSettings>() {
            app.world().resource::<MarketSettings>().clone()
        } else {
            let settings = load_market_settings();
            app.insert_resource(settings.clone());
            settings
        };

        app.init_database_table::<MarketPlayersTable>()
            .init_database_table::<MarketItemsTable>()
            .init_database_table::<MarketOffersTable>()
            .init_database_table::<MarketHistoryTable>();
        app.init_resource::<MarketDirty>();
        app.insert_resource(MarketFlushTimer(Timer::from_seconds(
            settings.persistence.flush_interval_secs.max(0.001) as f32,
            TimerMode::Repeating,
        )));

        if !app.world().contains_resource::<MarketOrmResource>() {
            let orm = build_market_orm(app.world(), &settings)
                .expect("Failed to build market ORM provider");
            app.insert_resource(MarketOrmResource::new(orm));
        }

        app.add_systems(Startup, load_market_tables_on_startup);
        app.add_systems(Update, autosave_market_tables);
        app.add_observer(save_market_tables_on_request)
            .add_observer(save_market_tables_on_shutdown);
    }
}

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

fn build_market_orm(
    _: &World,
    settings: &MarketSettings,
) -> anyhow::Result<std::sync::Arc<dyn MarketOrm>> {
    Ok(std::sync::Arc::new(SqlxMarketOrm::connect(
        &settings.persistence.database,
    )?))
}

fn load_market_tables_on_startup(
    orm: Option<Res<MarketOrmResource>>,
    mut players: DatabaseMut<MarketPlayersTable>,
    mut items: DatabaseMut<MarketItemsTable>,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
    mut history_sequence: ResMut<MarketHistorySequence>,
) {
    let Some(orm) = orm else {
        debug!(
            "MarketPersistencePlugin started without a MarketOrmResource; market tables stay \
             empty until populated by the app"
        );
        return;
    };

    match orm.load_players() {
        Ok(rows) => {
            let count = rows.len();
            players.replace(rows);
            info!("Loaded {count} market player names into MarketPlayersTable");
        }
        Err(error) => warn!("Failed to load market player names: {error:#}"),
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

    match orm.load_history() {
        Ok(rows) => {
            let count = rows.len();
            history.replace(rows);
            history_sequence.seed_from_history(&history);
            info!("Loaded {count} market history entries into MarketHistoryTable");
        }
        Err(error) => warn!("Failed to load market history entries: {error:#}"),
    }
}

#[allow(clippy::too_many_arguments)]
fn autosave_market_tables(
    time: Res<Time>,
    orm: Option<Res<MarketOrmResource>>,
    mut timer: ResMut<MarketFlushTimer>,
    mut dirty: ResMut<MarketDirty>,
    players: Database<MarketPlayersTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
    history: Database<MarketHistoryTable>,
) {
    if !dirty.0 {
        return;
    }

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Some(orm) = orm else {
        warn!("Buffered market writes are queued, but no MarketOrmResource is available");
        return;
    };

    if let Err(error) = persist_market_tables(orm.provider(), &players, &items, &offers, &history) {
        warn!("Failed to flush market tables: {error:#}");
        return;
    }

    dirty.0 = false;
}

fn save_market_tables_on_request(
    _: On<SaveMarketData>,
    orm: Option<Res<MarketOrmResource>>,
    mut dirty: ResMut<MarketDirty>,
    players: Database<MarketPlayersTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
    history: Database<MarketHistoryTable>,
) {
    if !dirty.0 {
        return;
    }

    let Some(orm) = orm else {
        warn!("Market save was requested, but no MarketOrmResource is available");
        return;
    };

    if let Err(error) = persist_market_tables(orm.provider(), &players, &items, &offers, &history) {
        warn!("Failed to save market tables on request: {error:#}");
        return;
    }

    dirty.0 = false;
}

#[allow(clippy::too_many_arguments)]
fn save_market_tables_on_shutdown(
    _: On<ShutdownMarketData>,
    settings: Res<MarketSettings>,
    orm: Option<Res<MarketOrmResource>>,
    mut dirty: ResMut<MarketDirty>,
    players: Database<MarketPlayersTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
    history: Database<MarketHistoryTable>,
) {
    if !settings.persistence.save_on_shutdown || !dirty.0 {
        return;
    }

    let Some(orm) = orm else {
        warn!("Market shutdown save was requested, but no MarketOrmResource is available");
        return;
    };

    if let Err(error) = persist_market_tables(orm.provider(), &players, &items, &offers, &history) {
        warn!("Failed to save market tables on shutdown: {error:#}");
        return;
    }

    dirty.0 = false;
}

fn persist_market_tables(
    orm: &dyn MarketOrm,
    players: &MarketPlayersTable,
    items: &MarketItemsTable,
    offers: &MarketOffersTable,
    history: &MarketHistoryTable,
) -> anyhow::Result<()> {
    orm.save_players(&players.rows())?;
    orm.save_items(&items.rows())?;
    orm.save_offers(&offers.rows())?;
    orm.save_history(&history.rows())?;
    Ok(())
}
