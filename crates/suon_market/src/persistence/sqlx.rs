use anyhow::{Context, Result};
use suon_database::prelude::*;

use crate::{
    history::{MarketHistoryAction, MarketHistoryEntry, MarketHistoryTable},
    offer::{
        MarketItem, MarketItemsTable, MarketOffer, MarketOfferId, MarketOffersTable,
        MarketPlayersTable, PlayerName,
    },
    persistence::orm::MarketOrm,
};

/// SQLx-backed market ORM that delegates connection/runtime concerns to
/// `suon_database` and keeps only market-specific table mappings here.
pub struct SqlxMarketOrm {
    database: DatabaseConnection<DatabasePool>,
    players: MarketPlayersMapper,
    items: MarketItemsMapper,
    offers: MarketOffersMapper,
    history: MarketHistoryMapper,
}

impl SqlxMarketOrm {
    pub fn connect(settings: &DatabaseSettings) -> Result<Self> {
        let database = DatabaseConnection::<DatabasePool>::connect(settings)?;
        let orm = Self {
            database,
            players: MarketPlayersMapper,
            items: MarketItemsMapper,
            offers: MarketOffersMapper,
            history: MarketHistoryMapper,
        };

        if settings.auto_initialize_schema() {
            MarketPlayersTable::default().initialize_schema(&orm.database, &orm.players)?;
            MarketItemsTable::default().initialize_schema(&orm.database, &orm.items)?;
            MarketOffersTable::default().initialize_schema(&orm.database, &orm.offers)?;
            MarketHistoryTable::default().initialize_schema(&orm.database, &orm.history)?;
        }

        Ok(orm)
    }
}

impl MarketOrm for SqlxMarketOrm {
    fn load_players(&self) -> Result<Vec<PlayerName>> {
        let mut table = MarketPlayersTable::default();
        table.load_from_database(&self.database, &self.players)?;
        Ok(table.rows())
    }

    fn load_items(&self) -> Result<Vec<MarketItem>> {
        let mut table = MarketItemsTable::default();
        table.load_from_database(&self.database, &self.items)?;
        Ok(table.rows())
    }

    fn load_offers(&self) -> Result<Vec<MarketOffer>> {
        let mut table = MarketOffersTable::default();
        table.load_from_database(&self.database, &self.offers)?;
        Ok(table.rows())
    }

    fn save_players(&self, players: &[PlayerName]) -> Result<()> {
        let mut table = MarketPlayersTable::default();
        table.replace(players.iter().cloned());
        table.save_to_database(&self.database, &self.players)?;
        Ok(())
    }

    fn save_items(&self, items: &[MarketItem]) -> Result<()> {
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

    fn load_history(&self) -> Result<Vec<MarketHistoryEntry>> {
        let mut table = MarketHistoryTable::default();
        table.load_from_database(&self.database, &self.history)?;
        Ok(table.rows())
    }

    fn save_history(&self, history: &[MarketHistoryEntry]) -> Result<()> {
        let mut table = MarketHistoryTable::default();
        table.replace(history.iter().cloned());
        table.save_to_database(&self.database, &self.history)?;
        Ok(())
    }
}

struct MarketPlayersMapper;

impl TableMapper<MarketPlayersTable, DatabasePool> for MarketPlayersMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS market_players (
                    id BIGINT PRIMARY KEY,
                    name TEXT NOT NULL
                )",
            )
            .execute(database.data().pool())
            .await
            .context("Failed to create market_players table")?;

            Ok(())
        })
    }

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<PlayerName>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketPlayerRow>("SELECT id, name FROM market_players")
                .fetch_all(database.data().pool())
                .await
                .context("Failed to load market players with SQLx")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[PlayerName],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_players")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_players before snapshot save")?;

            for record in rows.iter().map(MarketPlayerRow::from) {
                sqlx::query("INSERT INTO market_players (id, name) VALUES (?, ?)")
                    .bind(record.id)
                    .bind(record.name)
                    .execute(database.data().pool())
                    .await
                    .context("Failed to insert player snapshot with SQLx")?;
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

    fn load_rows(&self, database: &DatabaseConnection<DatabasePool>) -> Result<Vec<MarketItem>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketItemRow>("SELECT id, name FROM market_items")
                .fetch_all(database.data().pool())
                .await
                .context("Failed to load market items with SQLx")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketItem],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_items")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_items before snapshot save")?;

            for record in rows.iter().map(MarketItemRow::from) {
                sqlx::query("INSERT INTO market_items (id, name) VALUES (?, ?)")
                    .bind(record.id)
                    .bind(record.name)
                    .execute(database.data().pool())
                    .await
                    .context("Failed to insert item snapshot with SQLx")?;
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
                    player_id BIGINT NOT NULL,
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
                "SELECT timestamp_secs, counter, item_id, player_id, amount, price, side, \
                 is_anonymous FROM market_offers",
            )
            .fetch_all(database.data().pool())
            .await
            .context("Failed to load market offers with SQLx")?;

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
                sqlx::query(
                    "INSERT INTO market_offers (
                        timestamp_secs, counter, item_id, player_id, amount, price, side, \
                     is_anonymous
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(record.timestamp_secs)
                .bind(record.counter)
                .bind(record.item_id)
                .bind(record.player_id)
                .bind(record.amount)
                .bind(record.price)
                .bind(record.side)
                .bind(record.is_anonymous)
                .execute(database.data().pool())
                .await
                .context("Failed to insert offer snapshot with SQLx")?;
            }

            Ok(())
        })
    }
}

struct MarketHistoryMapper;

impl TableMapper<MarketHistoryTable, DatabasePool> for MarketHistoryMapper {
    fn initialize_schema(&self, database: &DatabaseConnection<DatabasePool>) -> Result<()> {
        database.block_on(async {
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS market_history (
                    id BIGINT PRIMARY KEY,
                    recorded_at_secs BIGINT NOT NULL,
                    action TEXT NOT NULL,
                    actor_player_id BIGINT NULL,
                    offer_player_id BIGINT NULL,
                    item_id INTEGER NULL,
                    offer_timestamp_secs BIGINT NULL,
                    offer_counter INTEGER NULL,
                    amount INTEGER NOT NULL,
                    remaining_amount INTEGER NULL,
                    price BIGINT NULL,
                    side TEXT NULL
                )",
            )
            .execute(database.data().pool())
            .await
            .context("Failed to create market_history table")?;

            Ok(())
        })
    }

    fn load_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
    ) -> Result<Vec<MarketHistoryEntry>> {
        database.block_on(async {
            let rows = sqlx::query_as::<_, MarketHistoryRow>(
                "SELECT id, recorded_at_secs, action, actor_player_id, offer_player_id, item_id,
                        offer_timestamp_secs, offer_counter, amount, remaining_amount, price, side
                 FROM market_history
                 ORDER BY id ASC",
            )
            .fetch_all(database.data().pool())
            .await
            .context("Failed to load market history with SQLx")?;

            rows.into_iter().map(TryInto::try_into).collect()
        })
    }

    fn save_rows(
        &self,
        database: &DatabaseConnection<DatabasePool>,
        rows: &[MarketHistoryEntry],
    ) -> Result<()> {
        database.block_on(async {
            sqlx::query("DELETE FROM market_history")
                .execute(database.data().pool())
                .await
                .context("Failed to clear market_history before snapshot save")?;

            for record in rows.iter().map(MarketHistoryRow::try_from) {
                let record = record?;
                sqlx::query(
                    "INSERT INTO market_history (
                        id, recorded_at_secs, action, actor_player_id, offer_player_id, item_id,
                        offer_timestamp_secs, offer_counter, amount, remaining_amount, price, side
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(record.id)
                .bind(record.recorded_at_secs)
                .bind(record.action)
                .bind(record.actor_player_id)
                .bind(record.offer_player_id)
                .bind(record.item_id)
                .bind(record.offer_timestamp_secs)
                .bind(record.offer_counter)
                .bind(record.amount)
                .bind(record.remaining_amount)
                .bind(record.price)
                .bind(record.side)
                .execute(database.data().pool())
                .await
                .context("Failed to insert market history snapshot with SQLx")?;
            }

            Ok(())
        })
    }
}

#[derive(sqlx::FromRow)]
struct MarketPlayerRow {
    id: i64,
    name: String,
}

impl From<&PlayerName> for MarketPlayerRow {
    fn from(player: &PlayerName) -> Self {
        Self {
            id: i64::from(player.id),
            name: player.name.clone(),
        }
    }
}

impl TryFrom<MarketPlayerRow> for PlayerName {
    type Error = anyhow::Error;

    fn try_from(row: MarketPlayerRow) -> Result<Self> {
        Ok(Self {
            id: row.id.try_u32_field("market_players.id")?,
            name: row.name,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MarketItemRow {
    id: i64,
    name: String,
}

impl From<&MarketItem> for MarketItemRow {
    fn from(item: &MarketItem) -> Self {
        Self {
            id: i64::from(item.id),
            name: item.name.clone(),
        }
    }
}

impl TryFrom<MarketItemRow> for MarketItem {
    type Error = anyhow::Error;

    fn try_from(row: MarketItemRow) -> Result<Self> {
        Ok(Self {
            id: row.id.try_u16_field("market_items.id")?,
            name: row.name,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MarketOfferRow {
    timestamp_secs: i64,
    counter: i64,
    item_id: i64,
    player_id: i64,
    amount: i64,
    price: i64,
    side: String,
    is_anonymous: bool,
}

impl TryFrom<&MarketOffer> for MarketOfferRow {
    type Error = anyhow::Error;

    fn try_from(offer: &MarketOffer) -> Result<Self> {
        Ok(Self {
            timestamp_secs: offer
                .id
                .timestamp
                .try_i64_secs_field("market_offers.timestamp_secs")?,
            counter: i64::from(offer.id.counter),
            item_id: i64::from(offer.item_id),
            player_id: i64::from(offer.player_id),
            amount: i64::from(offer.amount),
            price: offer.price.try_i64_field("market_offers.price")?,
            side: offer.side.to_string(),
            is_anonymous: offer.is_anonymous,
        })
    }
}

impl TryFrom<MarketOfferRow> for MarketOffer {
    type Error = anyhow::Error;

    fn try_from(row: MarketOfferRow) -> Result<Self> {
        Ok(Self {
            id: MarketOfferId {
                timestamp: std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(
                        row.timestamp_secs
                            .try_u64_field("market_offers.timestamp_secs")?,
                    ),
                counter: row.counter.try_u16_field("market_offers.counter")?,
            },
            item_id: row.item_id.try_u16_field("market_offers.item_id")?,
            player_id: row.player_id.try_u32_field("market_offers.player_id")?,
            amount: row.amount.try_u16_field("market_offers.amount")?,
            price: row.price.try_u64_field("market_offers.price")?,
            side: row.side.parse()?,
            is_anonymous: row.is_anonymous,
        })
    }
}

#[derive(sqlx::FromRow)]
struct MarketHistoryRow {
    id: i64,
    recorded_at_secs: i64,
    action: String,
    actor_player_id: Option<i64>,
    offer_player_id: Option<i64>,
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
        Ok(Self {
            id: entry.id.try_i64_field("market_history.id")?,
            recorded_at_secs: entry
                .recorded_at
                .try_i64_secs_field("market_history.recorded_at_secs")?,
            action: entry.action.to_string(),
            actor_player_id: entry.actor_player_id.map(i64::from),
            offer_player_id: entry.offer_player_id.map(i64::from),
            item_id: entry.item_id.map(i64::from),
            offer_timestamp_secs: entry
                .offer_id
                .map(|offer_id| {
                    offer_id
                        .timestamp
                        .try_i64_secs_field("market_history.offer_timestamp_secs")
                })
                .transpose()?,
            offer_counter: entry.offer_id.map(|offer_id| i64::from(offer_id.counter)),
            amount: i64::from(entry.amount),
            remaining_amount: entry.remaining_amount.map(i64::from),
            price: entry
                .price
                .map(|price| price.try_i64_field("market_history.price"))
                .transpose()?,
            side: entry.side.map(|side| side.to_string()),
        })
    }
}

impl TryFrom<MarketHistoryRow> for MarketHistoryEntry {
    type Error = anyhow::Error;

    fn try_from(row: MarketHistoryRow) -> Result<Self> {
        Ok(Self {
            id: row.id.try_u64_field("market_history.id")?,
            recorded_at: std::time::UNIX_EPOCH
                + std::time::Duration::from_secs(
                    row.recorded_at_secs
                        .try_u64_field("market_history.recorded_at_secs")?,
                ),
            action: row.action.parse::<MarketHistoryAction>()?,
            actor_player_id: row
                .actor_player_id
                .map(|value| value.try_u32_field("market_history.actor_player_id"))
                .transpose()?,
            offer_player_id: row
                .offer_player_id
                .map(|value| value.try_u32_field("market_history.offer_player_id"))
                .transpose()?,
            item_id: row
                .item_id
                .map(|value| value.try_u16_field("market_history.item_id"))
                .transpose()?,
            offer_id: match (row.offer_timestamp_secs, row.offer_counter) {
                (Some(timestamp_secs), Some(counter)) => Some(MarketOfferId {
                    timestamp: std::time::UNIX_EPOCH
                        + std::time::Duration::from_secs(
                            timestamp_secs.try_u64_field("market_history.offer_timestamp_secs")?,
                        ),
                    counter: counter.try_u16_field("market_history.offer_counter")?,
                }),
                (None, None) => None,
                _ => anyhow::bail!("market_history offer id columns must be both null or both set"),
            },
            amount: row.amount.try_u16_field("market_history.amount")?,
            remaining_amount: row
                .remaining_amount
                .map(|value| value.try_u16_field("market_history.remaining_amount"))
                .transpose()?,
            price: row
                .price
                .map(|value| value.try_u64_field("market_history.price"))
                .transpose()?,
            side: row.side.map(|value| value.parse()).transpose()?,
        })
    }
}
