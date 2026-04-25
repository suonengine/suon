//! Diesel-backed persistence mappings for the market domain.
//!
//! Unlike the previous Suon-specific ORM layer, this module follows Diesel's
//! native model style directly: `table!` schema declarations plus
//! `Queryable`/`Selectable`/`Insertable` structs.

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
    persistence::orm::MarketOrm,
};

/// Database-backed market ORM that delegates connection concerns to
/// `suon_database` and keeps market-specific Diesel mappings here.
pub struct MarketDatabaseOrm {
    database: DatabaseConnection<DatabasePool>,
    actors: MarketActorsMapper,
    items: MarketItemsMapper,
    offers: MarketOffersMapper,
    history: MarketHistoryMapper,
}

impl MarketDatabaseOrm {
    pub fn connect(settings: &DatabaseSettings) -> Result<Self> {
        let database = DatabaseConnection::<DatabasePool>::connect(settings)?;
        let orm = Self {
            database,
            actors: MarketActorsMapper,
            items: MarketItemsMapper,
            offers: MarketOffersMapper,
            history: MarketHistoryMapper,
        };

        if settings.auto_initialize_schema() {
            MarketActorsTable::default().initialize_schema(&orm.database, &orm.actors)?;
            MarketItemsTable::default().initialize_schema(&orm.database, &orm.items)?;
            MarketOffersTable::default().initialize_schema(&orm.database, &orm.offers)?;
            orm.history.initialize_schema(&orm.database)?;
        }

        Ok(orm)
    }
}

impl MarketOrm for MarketDatabaseOrm {
    fn load_actors(&self) -> Result<Vec<MarketActorName>> {
        let mut table = MarketActorsTable::default();
        table.load_from_database(&self.database, &self.actors)?;
        Ok(table.rows())
    }

    fn load_items(&self) -> Result<Vec<(u16, String)>> {
        let mut table = MarketItemsTable::default();
        table.load_from_database(&self.database, &self.items)?;
        Ok(table.rows())
    }

    fn load_offers(&self) -> Result<Vec<MarketOffer>> {
        let mut table = MarketOffersTable::default();
        table.load_from_database(&self.database, &self.offers)?;
        Ok(table.rows())
    }

    fn save_actors(&self, actors: &[MarketActorName]) -> Result<()> {
        let mut table = MarketActorsTable::default();
        table.replace(actors.iter().cloned());
        table.save_to_database(&self.database, &self.actors)?;
        Ok(())
    }

    fn save_items(&self, items: &[(u16, String)]) -> Result<()> {
        let mut table = MarketItemsTable::default();
        table.replace(items.iter().cloned());
        table.save_to_database(&self.database, &self.items)?;
        Ok(())
    }

    fn save_offers(&self, offers: &[MarketOffer]) -> Result<()> {
        let mut table = MarketOffersTable::default();
        table.replace(offers.iter().cloned());
        table.save_to_database(&self.database, &self.offers)?;
        Ok(())
    }

    fn insert_history(&self, entry: &MarketHistoryEntry) -> Result<()> {
        self.history.insert_row(&self.database, entry)
    }
}

struct MarketActorsMapper;

impl TableMapper<MarketActorsTable, DatabasePool> for MarketActorsMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.execute(|connection| {
            MarketActorRecord::ensure_table(connection, database.data().backend())
                .context("Failed to create market_actors table")?;

            Ok(())
        })
    }

    fn load_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
    ) -> Result<Vec<MarketActorName>> {
        database
            .execute(|connection| {
                connection
                    .query::<MarketActorRecord>()
                    .order(|actor| actor.id.asc())
                    .load()
            })
            .context("Failed to load market actors from the database")?
            .into_iter()
            .map(MarketActorRecord::into_domain)
            .collect()
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketActorName],
    ) -> Result<()> {
        let records = rows
            .iter()
            .map(MarketActorRecord::from_domain)
            .collect::<Vec<_>>();

        database.transaction(|connection| {
            connection
                .delete::<MarketActorRecord>()
                .execute()
                .context("Failed to clear market_actors before snapshot save")?;

            for record in &records {
                connection
                    .insert(record)
                    .execute()
                    .context("Failed to insert actor snapshot rows into the database")?;
            }

            Ok(())
        })
    }
}

/// Mapper responsible for snapshot persistence of market item names.
struct MarketItemsMapper;

impl TableMapper<MarketItemsTable, DatabasePool> for MarketItemsMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.execute(|connection| {
            MarketItemRecord::ensure_table(connection, database.data().backend())
                .context("Failed to create market_items table")?;

            Ok(())
        })
    }

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<(u16, String)>> {
        database
            .execute(|connection| {
                connection
                    .query::<MarketItemRecord>()
                    .order(|item| item.id.asc())
                    .load()
            })
            .context("Failed to load market items from the database")?
            .into_iter()
            .map(MarketItemRecord::into_domain)
            .collect()
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[(u16, String)],
    ) -> Result<()> {
        let records = rows
            .iter()
            .map(MarketItemRecord::from_domain)
            .collect::<Vec<_>>();

        database.transaction(|connection| {
            connection
                .delete::<MarketItemRecord>()
                .execute()
                .context("Failed to clear market_items before snapshot save")?;

            for record in &records {
                connection
                    .insert(record)
                    .execute()
                    .context("Failed to insert item snapshot rows into the database")?;
            }

            Ok(())
        })
    }
}

/// Mapper responsible for snapshot persistence of active market offers.
struct MarketOffersMapper;

impl TableMapper<MarketOffersTable, DatabasePool> for MarketOffersMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.execute(|connection| {
            MarketOfferRecord::ensure_table(connection, database.data().backend())
                .context("Failed to create market_offers table")?;

            Ok(())
        })
    }

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<MarketOffer>> {
        database
            .execute(|connection| {
                connection
                    .query::<MarketOfferRecord>()
                    .order(|offer| (offer.timestamp_secs.asc(), offer.counter.asc()))
                    .load()
            })
            .context("Failed to load market offers from the database")?
            .into_iter()
            .map(MarketOfferRecord::into_domain)
            .collect()
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketOffer],
    ) -> Result<()> {
        let records = rows
            .iter()
            .map(MarketOfferRecord::from_domain)
            .collect::<Result<Vec<_>>>()?;

        database.transaction(|connection| {
            connection
                .delete::<MarketOfferRecord>()
                .execute()
                .context("Failed to clear market_offers before snapshot save")?;

            for record in &records {
                connection
                    .insert(record)
                    .execute()
                    .context("Failed to insert offer snapshot rows into the database")?;
            }

            Ok(())
        })
    }
}

/// Mapper responsible for append-only history records.
struct MarketHistoryMapper;

impl MarketHistoryMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.execute(|connection| {
            MarketHistoryRecord::ensure_table(connection, database.data().backend())
                .context("Failed to create market_history table")?;

            Ok(())
        })
    }

    fn insert_row(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        entry: &MarketHistoryEntry,
    ) -> Result<()> {
        let record = NewMarketHistoryRecord::from_domain(entry)?;

        database.execute(|connection| {
            connection
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

/// Converts a history action enum into the persisted database representation.
fn serialize_history_action(action: MarketHistoryAction) -> String {
    action.to_string()
}

/// Converts a trade side enum into the persisted database representation.
fn serialize_trade_side(side: MarketTradeSide) -> String {
    side.to_string()
}
