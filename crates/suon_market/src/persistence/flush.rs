use bevy::{app::AppExit, prelude::*};
use log::warn;
use suon_database::prelude::*;

use crate::{
    offer::{MarketActorsTable, MarketItemsTable, MarketOffersTable},
    persistence::{MarketDirty, MarketOrm, MarketOrmResource, MarketSettings},
};

/// Timer used to flush dirty market tables on a fixed interval.
#[derive(Debug, Resource)]
pub(super) struct MarketFlushTimer(pub(super) Timer);

/// Persists the full in-memory market snapshot through the active ORM provider.
pub(super) fn persist_market_tables(
    orm: &dyn MarketOrm,
    actors: &MarketActorsTable,
    items: &MarketItemsTable,
    offers: &MarketOffersTable,
) -> anyhow::Result<()> {
    orm.save_actors(&actors.rows())?;
    orm.save_items(&items.rows())?;
    orm.save_offers(&offers.rows())?;
    Ok(())
}

/// Flushes dirty market tables on the configured autosave cadence.
#[allow(clippy::too_many_arguments)]
pub(super) fn autosave_market_tables(
    time: Res<Time>,
    orm: Option<Res<MarketOrmResource>>,
    mut timer: ResMut<MarketFlushTimer>,
    mut dirty: ResMut<MarketDirty>,
    actors: Database<MarketActorsTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
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

    if let Err(error) = persist_market_tables(orm.provider(), &actors, &items, &offers) {
        warn!("Failed to flush market tables: {error:#}");
        return;
    }

    dirty.clear();
}

/// Persists dirty market tables immediately when the app exits.
#[allow(clippy::too_many_arguments)]
pub(super) fn save_market_tables_on_app_exit(
    mut exits: MessageReader<AppExit>,
    settings: Res<MarketSettings>,
    orm: Option<Res<MarketOrmResource>>,
    mut dirty: ResMut<MarketDirty>,
    actors: Database<MarketActorsTable>,
    items: Database<MarketItemsTable>,
    offers: Database<MarketOffersTable>,
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

    if let Err(error) = persist_market_tables(orm.provider(), &actors, &items, &offers) {
        warn!("Failed to save market tables on app exit: {error:#}");
        return;
    }

    dirty.clear();
}
