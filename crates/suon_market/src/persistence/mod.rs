mod database;
mod orm;
mod settings;

use bevy::{app::AppExit, prelude::*};
use log::{debug, info, warn};
use suon_database::prelude::*;

use crate::{
    history::{MarketHistorySequence, MarketHistoryTable},
    offer::{MarketActorsTable, MarketItemsTable, MarketOfferSequence, MarketOffersTable},
};

pub use self::{
    orm::{MarketOrm, MarketOrmResource},
    settings::{MarketPersistenceSettings, MarketPolicySettings, MarketSettings},
};
pub(crate) use self::database::MarketDatabaseOrm;

#[derive(Debug, Resource)]
struct MarketFlushTimer(Timer);

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketDirty(pub(crate) bool);

impl MarketDirty {
    fn clear(&mut self) {
        self.0 = false;
    }

    pub(crate) fn mark(&mut self) {
        self.0 = true;
    }

    fn is_clean(&self) -> bool {
        !self.0
    }
}

pub(crate) struct MarketPersistencePlugin;

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

        app.init_database_table::<MarketActorsTable>()
            .init_database_table::<MarketItemsTable>()
            .init_database_table::<MarketOffersTable>()
            .init_database_table::<MarketHistoryTable>();
        app.init_resource::<MarketDirty>();
        app.insert_resource(MarketFlushTimer(Timer::from_seconds(
            settings.persistence().flush_interval_secs().max(0.001) as f32,
            TimerMode::Repeating,
        )));

        if !app.world().contains_resource::<MarketOrmResource>() {
            let orm = build_market_orm(app.world(), &settings)
                .expect("Failed to build market ORM provider");
            app.insert_resource(MarketOrmResource::new(orm));
        }

        app.add_systems(Startup, load_market_tables_on_startup);
        app.add_systems(Update, (autosave_market_tables, save_market_tables_on_app_exit));
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
    Ok(std::sync::Arc::new(MarketDatabaseOrm::connect(
        settings.persistence().database(),
    )?))
}

fn persist_market_tables(
    orm: &dyn MarketOrm,
    actors: &MarketActorsTable,
    items: &MarketItemsTable,
    offers: &MarketOffersTable,
    history: &MarketHistoryTable,
) -> anyhow::Result<()> {
    orm.save_actors(&actors.rows())?;
    orm.save_items(&items.rows())?;
    orm.save_offers(&offers.rows())?;
    orm.save_history(&history.rows())?;
    Ok(())
}

fn load_market_tables_on_startup(
    orm: Option<Res<MarketOrmResource>>,
    mut actors: DatabaseMut<MarketActorsTable>,
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
    actors: Database<MarketActorsTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
    history: Database<MarketHistoryTable>,
) {
    if dirty.is_clean() {
        return;
    }

    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Some(orm) = orm else {
        warn!("Buffered market writes are queued, but no MarketOrmResource is available");
        return;
    };

    if let Err(error) = persist_market_tables(orm.provider(), &actors, &items, &offers, &history) {
        warn!("Failed to flush market tables: {error:#}");
        return;
    }

    dirty.clear();
}

#[allow(clippy::too_many_arguments)]
fn save_market_tables_on_app_exit(
    mut exits: MessageReader<AppExit>,
    settings: Res<MarketSettings>,
    orm: Option<Res<MarketOrmResource>>,
    mut dirty: ResMut<MarketDirty>,
    actors: Database<MarketActorsTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
    history: Database<MarketHistoryTable>,
) {
    if exits.read().last().is_none() {
        return;
    }

    if !settings.persistence().save_on_shutdown() || dirty.is_clean() {
        return;
    }

    let Some(orm) = orm else {
        warn!("Market app exit was requested, but no MarketOrmResource is available");
        return;
    };

    if let Err(error) = persist_market_tables(orm.provider(), &actors, &items, &offers, &history) {
        warn!("Failed to save market tables on app exit: {error:#}");
        return;
    }

    dirty.clear();
}
