//! Diesel-backed persistence for the market domain.
//!
//! Each market table implements [`DbTable`] directly: no separate mapper, no
//! ORM trait. The history journal implements [`DbAppend`]. The persistence
//! pipeline registered by `suon_database` then loads, saves, and drains
//! tables automatically based on their dirty epoch.

use anyhow::{Context, Result};
use diesel::{ExpressionMethods, Insertable};
use suon_database::prelude::*;
use suon_macros::database_model;

use crate::{
    history::{MarketHistoryAction, MarketHistoryEntry},
    offer::{
        MarketActorName, MarketActorsTable, MarketItemsTable, MarketOffer, MarketOfferId,
        MarketOffersTable, MarketTradeSide,
    },
};

impl DbTable for MarketActorsTable {
    type Row = MarketActorName;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketActorsTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketActorsTable::rows(self)
    }

    fn initialize_schema(connection: &DbConnection) -> Result<()> {
        connection.execute(|driver| {
            MarketActorRecord::ensure_table(driver, driver.backend())
                .context("Failed to create market_actors table")?;
            Ok(())
        })
    }

    fn load(connection: &DbConnection) -> Result<Vec<Self::Row>> {
        connection
            .execute(|driver| {
                driver
                    .query::<MarketActorRecord>()
                    .order(|actor| actor.id.asc())
                    .load()
            })
            .context("Failed to load market actors from the database")?
            .into_iter()
            .map(MarketActorRecord::into_domain)
            .collect()
    }

    fn save(connection: &DbConnection, rows: &[Self::Row]) -> Result<()> {
        let records: Vec<_> = rows.iter().map(MarketActorRecord::from_domain).collect();

        connection.transaction(|driver| {
            driver
                .delete::<MarketActorRecord>()
                .execute()
                .context("Failed to clear market_actors before snapshot save")?;

            for record in &records {
                driver
                    .insert(record)
                    .execute()
                    .context("Failed to insert actor snapshot row")?;
            }

            Ok(())
        })
    }
}

impl DbTable for MarketItemsTable {
    type Row = (u16, String);

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketItemsTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketItemsTable::rows(self)
    }

    fn initialize_schema(connection: &DbConnection) -> Result<()> {
        connection.execute(|driver| {
            MarketItemRecord::ensure_table(driver, driver.backend())
                .context("Failed to create market_items table")?;
            Ok(())
        })
    }

    fn load(connection: &DbConnection) -> Result<Vec<Self::Row>> {
        connection
            .execute(|driver| {
                driver
                    .query::<MarketItemRecord>()
                    .order(|item| item.id.asc())
                    .load()
            })
            .context("Failed to load market items from the database")?
            .into_iter()
            .map(MarketItemRecord::into_domain)
            .collect()
    }

    fn save(connection: &DbConnection, rows: &[Self::Row]) -> Result<()> {
        let records: Vec<_> = rows.iter().map(MarketItemRecord::from_domain).collect();

        connection.transaction(|driver| {
            driver
                .delete::<MarketItemRecord>()
                .execute()
                .context("Failed to clear market_items before snapshot save")?;

            for record in &records {
                driver
                    .insert(record)
                    .execute()
                    .context("Failed to insert item snapshot row")?;
            }

            Ok(())
        })
    }
}

impl DbTable for MarketOffersTable {
    type Row = MarketOffer;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketOffersTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketOffersTable::rows(self)
    }

    fn initialize_schema(connection: &DbConnection) -> Result<()> {
        connection.execute(|driver| {
            MarketOfferRecord::ensure_table(driver, driver.backend())
                .context("Failed to create market_offers table")?;
            Ok(())
        })
    }

    fn load(connection: &DbConnection) -> Result<Vec<Self::Row>> {
        connection
            .execute(|driver| {
                driver
                    .query::<MarketOfferRecord>()
                    .order(|offer| (offer.timestamp_secs.asc(), offer.counter.asc()))
                    .load()
            })
            .context("Failed to load market offers from the database")?
            .into_iter()
            .map(MarketOfferRecord::into_domain)
            .collect()
    }

    fn save(connection: &DbConnection, rows: &[Self::Row]) -> Result<()> {
        let records: Vec<_> = rows
            .iter()
            .map(MarketOfferRecord::from_domain)
            .collect::<Result<_>>()?;

        connection.transaction(|driver| {
            driver
                .delete::<MarketOfferRecord>()
                .execute()
                .context("Failed to clear market_offers before snapshot save")?;

            for record in &records {
                driver
                    .insert(record)
                    .execute()
                    .context("Failed to insert offer snapshot row")?;
            }

            Ok(())
        })
    }
}

/// Append-only journal for market history events.
pub struct MarketHistoryJournal;

impl DbAppend for MarketHistoryJournal {
    type Row = MarketHistoryEntry;

    fn initialize_schema(connection: &DbConnection) -> Result<()> {
        connection.execute(|driver| {
            MarketHistoryRecord::ensure_table(driver, driver.backend())
                .context("Failed to create market_history table")?;
            Ok(())
        })
    }

    fn append(connection: &DbConnection, entry: &Self::Row) -> Result<()> {
        let record = NewMarketHistoryRecord::from_domain(entry)?;

        connection.execute(|driver| {
            driver
                .insert(&record)
                .execute()
                .context("Failed to append market history entry")?;
            Ok(())
        })
    }
}

/// Database record used for both reading and replacing actor snapshots.
#[database_model(table = "market_actors")]
#[derive(Debug, Clone)]
struct MarketActorRecord {
    #[database(primary_key)]
    pub id: i64,
    pub name: String,
}

impl MarketActorRecord {
    fn into_domain(self) -> Result<MarketActorName> {
        Ok(MarketActorName::new(
            self.id.try_field("market_actors.id")?,
            self.name,
        ))
    }

    fn from_domain(actor: &MarketActorName) -> Self {
        Self {
            id: i64::from(actor.id()),
            name: actor.name().to_owned(),
        }
    }
}

/// Database record used for both reading and replacing item snapshots.
#[database_model(table = "market_items")]
#[derive(Debug, Clone)]
struct MarketItemRecord {
    #[database(primary_key)]
    pub id: i32,
    pub name: String,
}

impl MarketItemRecord {
    fn into_domain(self) -> Result<(u16, String)> {
        Ok((self.id.try_field("market_items.id")?, self.name))
    }

    fn from_domain(item: &(u16, String)) -> Self {
        Self {
            id: i32::from(item.0),
            name: item.1.clone(),
        }
    }
}

/// Database record used for both reading and replacing offer snapshots.
#[database_model(table = "market_offers")]
#[derive(Debug, Clone)]
struct MarketOfferRecord {
    #[database(primary_key)]
    pub timestamp_secs: i64,
    #[database(primary_key)]
    pub counter: i32,
    pub item_id: i32,
    pub actor_id: i64,
    pub amount: i32,
    pub price: i64,
    pub side: String,
    pub is_anonymous: i32,
}

impl MarketOfferRecord {
    fn into_domain(self) -> Result<MarketOffer> {
        Ok(MarketOffer::new(
            MarketOfferId::new(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(
                        self.timestamp_secs
                            .try_field("market_offers.timestamp_secs")?,
                    ),
                self.counter.try_field("market_offers.counter")?,
            ),
            self.item_id.try_field("market_offers.item_id")?,
            self.actor_id.try_field("market_offers.actor_id")?,
            self.amount.try_field("market_offers.amount")?,
            self.price.try_field("market_offers.price")?,
            self.side.parse()?,
            self.is_anonymous != 0,
        ))
    }

    fn from_domain(offer: &MarketOffer) -> Result<Self> {
        let offer_id = offer.id();
        Ok(Self {
            timestamp_secs: offer_id
                .timestamp()
                .try_i64_secs_field("market_offers.timestamp_secs")?,
            counter: i32::from(offer_id.counter()),
            item_id: i32::from(offer.item_id()),
            actor_id: i64::from(offer.actor_id()),
            amount: i32::from(offer.amount()),
            price: offer.price().try_field("market_offers.price")?,
            side: offer.side().to_string(),
            is_anonymous: if offer.is_anonymous() { 1 } else { 0 },
        })
    }
}

/// Row model that defines the append-only market history schema.
#[database_model(table = "market_history")]
#[derive(Debug, Clone)]
struct MarketHistoryRecord {
    #[database(primary_key, auto)]
    pub id: i64,
    pub recorded_at_secs: i64,
    pub action: String,
    pub actor_id: Option<i64>,
    pub offer_actor_id: Option<i64>,
    pub item_id: Option<i32>,
    pub offer_timestamp_secs: Option<i64>,
    pub offer_counter: Option<i32>,
    pub amount: i32,
    pub remaining_amount: Option<i32>,
    pub price: Option<i64>,
    pub side: Option<String>,
}

/// Insert model for append-only market history rows.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = market_history)]
struct NewMarketHistoryRecord {
    pub recorded_at_secs: i64,
    pub action: String,
    pub actor_id: Option<i64>,
    pub offer_actor_id: Option<i64>,
    pub item_id: Option<i32>,
    pub offer_timestamp_secs: Option<i64>,
    pub offer_counter: Option<i32>,
    pub amount: i32,
    pub remaining_amount: Option<i32>,
    pub price: Option<i64>,
    pub side: Option<String>,
}

impl diesel::associations::HasTable for NewMarketHistoryRecord {
    type Table = market_history::table;

    fn table() -> Self::Table {
        market_history::table
    }
}

impl NewMarketHistoryRecord {
    fn from_domain(entry: &MarketHistoryEntry) -> Result<Self> {
        let offer_id = entry.offer_id();
        Ok(Self {
            recorded_at_secs: entry
                .recorded_at()
                .try_i64_secs_field("market_history.recorded_at_secs")?,
            action: serialize_history_action(entry.action()),
            actor_id: entry.actor_id().map(i64::from),
            offer_actor_id: entry.offer_actor_id().map(i64::from),
            item_id: entry.item_id().map(i32::from),
            offer_timestamp_secs: offer_id
                .map(|offer_id| {
                    offer_id
                        .timestamp()
                        .try_i64_secs_field("market_history.offer_timestamp_secs")
                })
                .transpose()?,
            offer_counter: offer_id.map(|offer_id| i32::from(offer_id.counter())),
            amount: i32::from(entry.amount()),
            remaining_amount: entry.remaining_amount().map(i32::from),
            price: entry
                .price()
                .map(|price| price.try_field("market_history.price"))
                .transpose()?,
            side: entry.side().map(serialize_trade_side),
        })
    }
}

fn serialize_history_action(action: MarketHistoryAction) -> String {
    action.to_string()
}

fn serialize_trade_side(side: MarketTradeSide) -> String {
    side.to_string()
}
