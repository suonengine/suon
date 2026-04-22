use anyhow::{Context, Result};
use suon_database::prelude::*;

use crate::{
    history::MarketHistoryEntry,
    offer::{
        MarketActorName, MarketActorsTable, MarketItemsTable, MarketOffer, MarketOfferId,
        MarketOffersTable,
    },
    persistence::orm::MarketOrm,
};

/// Database-backed market ORM that delegates connection/runtime concerns to
/// `suon_database` and keeps only market-specific table mappings here.
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
        database.block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS market_actors (
                    id BIGINT PRIMARY KEY,
                    name TEXT NOT NULL
                )",
            )
            .execute(database.data().pool())
            .await
            .context("Failed to create market_actors table")?;

            Ok(())
        })
    }

    fn load_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
    ) -> Result<Vec<MarketActorName>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketActorRow>("SELECT id, name FROM market_actors")
                .fetch_all(database.data().pool())
                .await
                .context("Failed to load market actors from the database")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketActorName],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_actors")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_actors before snapshot save")?;

            for record in rows.iter().map(MarketActorRow::from) {
                sqlx::query(insert_market_actor_sql(database.data().backend()))
                    .bind(record.id)
                    .bind(record.name)
                    .execute(database.data().pool())
                    .await
                    .context("Failed to insert actor snapshot into the database")?;
            }

            Ok(())
        })
    }
}

struct MarketItemsMapper;

impl TableMapper<MarketItemsTable, DatabasePool> for MarketItemsMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS market_items (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                )",
            )
            .execute(database.data().pool())
            .await
            .context("Failed to create market_items table")?;

            Ok(())
        })
    }

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<(u16, String)>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketItemRow>("SELECT id, name FROM market_items")
                .fetch_all(database.data().pool())
                .await
                .context("Failed to load market items from the database")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[(u16, String)],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_items")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_items before snapshot save")?;

            for record in rows.iter().map(MarketItemRow::from) {
                sqlx::query(insert_market_item_sql(database.data().backend()))
                    .bind(record.id)
                    .bind(record.name)
                    .execute(database.data().pool())
                    .await
                    .context("Failed to insert item snapshot into the database")?;
            }

            Ok(())
        })
    }
}

struct MarketOffersMapper;

impl TableMapper<MarketOffersTable, DatabasePool> for MarketOffersMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS market_offers (
                    timestamp_secs BIGINT NOT NULL,
                    counter INTEGER NOT NULL,
                    item_id INTEGER NOT NULL,
                    actor_id BIGINT NOT NULL,
                    amount INTEGER NOT NULL,
                    price BIGINT NOT NULL,
                    side TEXT NOT NULL,
                    is_anonymous BOOLEAN NOT NULL,
                    PRIMARY KEY (timestamp_secs, counter)
                )",
            )
            .execute(database.data().pool())
            .await
            .context("Failed to create market_offers table")?;

            Ok(())
        })
    }

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<MarketOffer>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketOfferRow>(
                "SELECT timestamp_secs, counter, item_id, actor_id, amount, price, side, \
                 is_anonymous FROM market_offers",
            )
            .fetch_all(database.data().pool())
            .await
            .context("Failed to load market offers from the database")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketOffer],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_offers")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_offers before snapshot save")?;

            for record in rows.iter().map(MarketOfferRow::try_from) {
                let record = record?;
                sqlx::query(insert_market_offer_sql(database.data().backend()))
                    .bind(record.timestamp_secs)
                    .bind(record.counter)
                    .bind(record.item_id)
                    .bind(record.actor_id)
                    .bind(record.amount)
                    .bind(record.price)
                    .bind(record.side)
                    .bind(record.is_anonymous)
                    .execute(database.data().pool())
                    .await
                    .context("Failed to insert offer snapshot into the database")?;
            }

            Ok(())
        })
    }
}

struct MarketHistoryMapper;

impl MarketHistoryMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.block_on(async {
            sqlx::query(create_market_history_sql(database.data().backend()))
                .execute(database.data().pool())
                .await
                .context("Failed to create market_history table")?;

            Ok(())
        })
    }

    fn insert_row(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        entry: &MarketHistoryEntry,
    ) -> Result<()> {
        let record = MarketHistoryRow::try_from(entry)?;

        database.block_on(async {
            sqlx::query(insert_market_history_sql(database.data().backend()))
                .bind(record.recorded_at_secs)
                .bind(record.action)
                .bind(record.actor_id)
                .bind(record.offer_actor_id)
                .bind(record.item_id)
                .bind(record.offer_timestamp_secs)
                .bind(record.offer_counter)
                .bind(record.amount)
                .bind(record.remaining_amount)
                .bind(record.price)
                .bind(record.side)
                .execute(database.data().pool())
                .await
                .context("Failed to append market history entry")?;

            Ok(())
        })
    }
}

fn create_market_history_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Sqlite => {
            "CREATE TABLE IF NOT EXISTS market_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at_secs BIGINT NOT NULL,
                action TEXT NOT NULL,
                actor_id BIGINT NULL,
                offer_actor_id BIGINT NULL,
                item_id INTEGER NULL,
                offer_timestamp_secs BIGINT NULL,
                offer_counter INTEGER NULL,
                amount INTEGER NOT NULL,
                remaining_amount INTEGER NULL,
                price BIGINT NULL,
                side TEXT NULL
            )"
        }
        DatabaseBackend::Postgres => {
            "CREATE TABLE IF NOT EXISTS market_history (
                id BIGSERIAL PRIMARY KEY,
                recorded_at_secs BIGINT NOT NULL,
                action TEXT NOT NULL,
                actor_id BIGINT NULL,
                offer_actor_id BIGINT NULL,
                item_id INTEGER NULL,
                offer_timestamp_secs BIGINT NULL,
                offer_counter INTEGER NULL,
                amount INTEGER NOT NULL,
                remaining_amount INTEGER NULL,
                price BIGINT NULL,
                side TEXT NULL
            )"
        }
        DatabaseBackend::MySql | DatabaseBackend::MariaDb => {
            "CREATE TABLE IF NOT EXISTS market_history (
                id BIGINT AUTO_INCREMENT PRIMARY KEY,
                recorded_at_secs BIGINT NOT NULL,
                action TEXT NOT NULL,
                actor_id BIGINT NULL,
                offer_actor_id BIGINT NULL,
                item_id INTEGER NULL,
                offer_timestamp_secs BIGINT NULL,
                offer_counter INTEGER NULL,
                amount INTEGER NOT NULL,
                remaining_amount INTEGER NULL,
                price BIGINT NULL,
                side TEXT NULL
            )"
        }
    }
}

fn insert_market_actor_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Postgres => "INSERT INTO market_actors (id, name) VALUES ($1, $2)",
        DatabaseBackend::Sqlite | DatabaseBackend::MySql | DatabaseBackend::MariaDb => {
            "INSERT INTO market_actors (id, name) VALUES (?, ?)"
        }
    }
}

fn insert_market_item_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Postgres => "INSERT INTO market_items (id, name) VALUES ($1, $2)",
        DatabaseBackend::Sqlite | DatabaseBackend::MySql | DatabaseBackend::MariaDb => {
            "INSERT INTO market_items (id, name) VALUES (?, ?)"
        }
    }
}

fn insert_market_offer_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Postgres => {
            "INSERT INTO market_offers (
                timestamp_secs, counter, item_id, actor_id, amount, price, side, is_anonymous
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        }
        DatabaseBackend::Sqlite | DatabaseBackend::MySql | DatabaseBackend::MariaDb => {
            "INSERT INTO market_offers (
                timestamp_secs, counter, item_id, actor_id, amount, price, side, is_anonymous
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        }
    }
}

fn insert_market_history_sql(backend: DatabaseBackend) -> &'static str {
    match backend {
        DatabaseBackend::Postgres => {
            "INSERT INTO market_history (
                recorded_at_secs, action, actor_id, offer_actor_id, item_id,
                offer_timestamp_secs, offer_counter, amount, remaining_amount, price, side
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        }
        DatabaseBackend::Sqlite | DatabaseBackend::MySql | DatabaseBackend::MariaDb => {
            "INSERT INTO market_history (
                recorded_at_secs, action, actor_id, offer_actor_id, item_id,
                offer_timestamp_secs, offer_counter, amount, remaining_amount, price, side
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        }
    }
}

#[derive(sqlx::FromRow)]
struct MarketActorRow {
    id: i64,
    name: String,
}

impl From<&MarketActorName> for MarketActorRow {
    fn from(actor: &MarketActorName) -> Self {
        Self {
            id: i64::from(actor.id()),
            name: actor.name().to_owned(),
        }
    }
}

impl TryFrom<MarketActorRow> for MarketActorName {
    type Error = anyhow::Error;

    fn try_from(row: MarketActorRow) -> Result<Self> {
        Ok(Self::new(row.id.try_field("market_actors.id")?, row.name))
    }
}

#[derive(sqlx::FromRow)]
struct MarketItemRow {
    id: i64,
    name: String,
}

impl From<&(u16, String)> for MarketItemRow {
    fn from(item: &(u16, String)) -> Self {
        Self {
            id: i64::from(item.0),
            name: item.1.clone(),
        }
    }
}

impl TryFrom<MarketItemRow> for (u16, String) {
    type Error = anyhow::Error;

    fn try_from(row: MarketItemRow) -> Result<Self> {
        Ok((row.id.try_field("market_items.id")?, row.name))
    }
}

#[derive(sqlx::FromRow)]
struct MarketOfferRow {
    timestamp_secs: i64,
    counter: i64,
    item_id: i64,
    actor_id: i64,
    amount: i64,
    price: i64,
    side: String,
    is_anonymous: bool,
}

impl TryFrom<&MarketOffer> for MarketOfferRow {
    type Error = anyhow::Error;

    fn try_from(offer: &MarketOffer) -> Result<Self> {
        let offer_id = offer.id();
        Ok(Self {
            timestamp_secs: offer_id
                .timestamp()
                .try_i64_secs_field("market_offers.timestamp_secs")?,
            counter: i64::from(offer_id.counter()),
            item_id: i64::from(offer.item_id()),
            actor_id: i64::from(offer.actor_id()),
            amount: i64::from(offer.amount()),
            price: offer.price().try_field("market_offers.price")?,
            side: offer.side().to_string(),
            is_anonymous: offer.is_anonymous(),
        })
    }
}

impl TryFrom<MarketOfferRow> for MarketOffer {
    type Error = anyhow::Error;

    fn try_from(row: MarketOfferRow) -> Result<Self> {
        Ok(Self::new(
            MarketOfferId::new(
                std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(
                        row.timestamp_secs
                            .try_field("market_offers.timestamp_secs")?,
                    ),
                row.counter.try_field("market_offers.counter")?,
            ),
            row.item_id.try_field("market_offers.item_id")?,
            row.actor_id.try_field("market_offers.actor_id")?,
            row.amount.try_field("market_offers.amount")?,
            row.price.try_field("market_offers.price")?,
            row.side.parse()?,
            row.is_anonymous,
        ))
    }
}

struct MarketHistoryRow {
    recorded_at_secs: i64,
    action: String,
    actor_id: Option<i64>,
    offer_actor_id: Option<i64>,
    item_id: Option<i64>,
    offer_timestamp_secs: Option<i64>,
    offer_counter: Option<i64>,
    amount: i64,
    remaining_amount: Option<i64>,
    price: Option<i64>,
    side: Option<String>,
}

impl TryFrom<&MarketHistoryEntry> for MarketHistoryRow {
    type Error = anyhow::Error;

    fn try_from(entry: &MarketHistoryEntry) -> Result<Self> {
        let offer_id = entry.offer_id();
        Ok(Self {
            recorded_at_secs: entry
                .recorded_at()
                .try_i64_secs_field("market_history.recorded_at_secs")?,
            action: entry.action().to_string(),
            actor_id: entry.actor_id().map(i64::from),
            offer_actor_id: entry.offer_actor_id().map(i64::from),
            item_id: entry.item_id().map(i64::from),
            offer_timestamp_secs: offer_id
                .map(|offer_id| {
                    offer_id
                        .timestamp()
                        .try_i64_secs_field("market_history.offer_timestamp_secs")
                })
                .transpose()?,
            offer_counter: offer_id.map(|offer_id| i64::from(offer_id.counter())),
            amount: i64::from(entry.amount()),
            remaining_amount: entry.remaining_amount().map(i64::from),
            price: entry
                .price()
                .map(|price| price.try_field("market_history.price"))
                .transpose()?,
            side: entry.side().map(|side| side.to_string()),
        })
    }
}
