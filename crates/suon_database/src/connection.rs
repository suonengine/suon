//! Backend, driver, and shared connection used by every persistence layer.
//!
//! This module collapses what used to live in multiple files:
//! `DatabaseBackend`, `DatabaseDriverConnection`, `DatabasePool`, and
//! `DatabaseConnection`. Callers now work with two clear types:
//!
//! - [`DbDriver`]: the multi-backend Diesel connection, exposed when raw SQL
//!   is needed.
//! - [`DbConnection`]: a Bevy [`Resource`] holding the driver behind a mutex
//!   and offering `execute` / `transaction` / `block_on` helpers.
//!
//! [`Resource`]: bevy::prelude::Resource

use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Context, Result, anyhow, bail};
use bevy::{prelude::*, tasks::block_on};
#[cfg(feature = "mysql")]
use diesel::MysqlConnection;
#[cfg(feature = "postgres")]
use diesel::PgConnection;
#[cfg(feature = "sqlite")]
use diesel::SqliteConnection;
use diesel::{
    Connection, RunQueryDsl, associations::HasTable, query_builder::IntoUpdateTarget, sql_query,
};

use crate::{
    record::{DbRecord, PendingInsert, PendingStatement},
    settings::DbSettings,
};

#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
compile_error!("Enable at least one suon_database backend feature: sqlite, postgres, or mysql.");

/// Database backend supported by the enabled integration features.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DbBackend {
    /// SQLite file or in-memory backend.
    Sqlite,
    /// PostgreSQL backend.
    Postgres,
    /// MySQL backend.
    MySql,
    /// MariaDB backend routed through MySQL support.
    MariaDb,
}

impl DbBackend {
    /// Detects the database backend from a configured URL.
    pub fn from_url(database_url: &str) -> Result<Self> {
        if database_url.starts_with("sqlite:") {
            #[cfg(feature = "sqlite")]
            {
                return Ok(Self::Sqlite);
            }
            #[cfg(not(feature = "sqlite"))]
            {
                bail!("SQLite database URL was configured, but the 'sqlite' feature is disabled");
            }
        }

        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            #[cfg(feature = "postgres")]
            {
                return Ok(Self::Postgres);
            }
            #[cfg(not(feature = "postgres"))]
            {
                bail!(
                    "PostgreSQL database URL was configured, but the 'postgres' feature is \
                     disabled"
                );
            }
        }

        if database_url.starts_with("mysql://") {
            #[cfg(feature = "mysql")]
            {
                return Ok(Self::MySql);
            }
            #[cfg(not(feature = "mysql"))]
            {
                bail!("MySQL database URL was configured, but the 'mysql' feature is disabled");
            }
        }

        if database_url.starts_with("mariadb://") {
            #[cfg(feature = "mysql")]
            {
                return Ok(Self::MariaDb);
            }
            #[cfg(not(feature = "mysql"))]
            {
                bail!("MariaDB database URL was configured, but the 'mysql' feature is disabled");
            }
        }

        bail!("Unsupported database URL scheme for enabled database backends: '{database_url}'")
    }
}

/// Multi-backend Diesel connection used across all enabled features.
#[derive(diesel::MultiConnection)]
pub enum DbDriver {
    /// PostgreSQL connection variant.
    #[cfg(feature = "postgres")]
    Postgres(PgConnection),
    /// MySQL or MariaDB connection variant.
    #[cfg(feature = "mysql")]
    Mysql(MysqlConnection),
    /// SQLite connection variant available in the default feature set.
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
}

impl DbDriver {
    /// Returns the backend represented by this active driver.
    pub fn backend(&self) -> DbBackend {
        match self {
            #[cfg(feature = "postgres")]
            Self::Postgres(_) => DbBackend::Postgres,
            #[cfg(feature = "mysql")]
            Self::Mysql(_) => DbBackend::MySql,
            #[cfg(feature = "sqlite")]
            Self::Sqlite(_) => DbBackend::Sqlite,
        }
    }

    /// Builds a typed select statement inferred from the record's table.
    pub fn query<Record>(&mut self) -> PendingStatement<'_, Record::Query, Record>
    where
        Record: DbRecord,
    {
        PendingStatement::new(self, Record::query())
    }

    /// Builds a pending insert inferred from the record's table.
    pub fn insert<Record>(&mut self, record: Record) -> PendingInsert<'_, Record>
    where
        Record: HasTable,
    {
        PendingInsert::new(self, record)
    }

    /// Builds a delete statement inferred from the record's table.
    pub fn delete<Record>(&mut self) -> PendingStatement<'_, diesel::dsl::delete<Record::Table>>
    where
        Record: HasTable,
        Record::Table: IntoUpdateTarget,
    {
        PendingStatement::new(self, diesel::delete(Record::table()))
    }
}

/// Bevy [`Resource`] holding the active driver behind a mutex.
///
/// `DbConnection` is the single connection abstraction exposed to gameplay
/// systems. Clone freely: the underlying driver is reference-counted, so all
/// clones share the same locked state and background tasks can take an owned
/// copy without giving up the live resource.
///
/// [`Resource`]: bevy::prelude::Resource
#[derive(Clone, Resource)]
pub struct DbConnection {
    driver: Arc<Mutex<DbDriver>>,
}

impl DbConnection {
    /// Opens a database connection from the supplied settings.
    pub fn open(settings: &DbSettings) -> Result<Self> {
        let backend = DbBackend::from_url(settings.database_url())?;

        if backend == DbBackend::Sqlite {
            ensure_sqlite_parent_dir(settings)?;
        }

        let normalized_url = settings.normalized_database_url();
        let driver = match backend {
            #[cfg(feature = "sqlite")]
            DbBackend::Sqlite => DbDriver::Sqlite(
                SqliteConnection::establish(&sqlite_connection_target(&normalized_url)?)
                    .with_context(|| {
                        format!(
                            "Failed to establish SQLite connection for '{}'",
                            settings.database_url()
                        )
                    })?,
            ),
            #[cfg(feature = "postgres")]
            DbBackend::Postgres => DbDriver::Postgres(
                PgConnection::establish(&normalized_url).with_context(|| {
                    format!(
                        "Failed to establish PostgreSQL connection for '{}'",
                        settings.database_url()
                    )
                })?,
            ),
            #[cfg(feature = "mysql")]
            DbBackend::MySql | DbBackend::MariaDb => DbDriver::Mysql(
                MysqlConnection::establish(&normalized_url).with_context(|| {
                    format!(
                        "Failed to establish MySQL/MariaDB connection for '{}'",
                        settings.database_url()
                    )
                })?,
            ),
            #[allow(unreachable_patterns)]
            backend => bail!("Database backend {backend:?} is disabled by crate features"),
        };

        let connection = Self {
            driver: Arc::new(Mutex::new(driver)),
        };

        #[cfg(feature = "sqlite")]
        if backend == DbBackend::Sqlite {
            connection.execute(|driver| configure_sqlite_driver(driver, settings))?;
        }

        Ok(connection)
    }

    /// Runs the closure with exclusive mutable access to the underlying driver.
    pub fn execute<T>(&self, work: impl FnOnce(&mut DbDriver) -> Result<T>) -> Result<T> {
        let mut driver = self.lock_driver()?;
        work(&mut driver)
    }

    /// Runs the closure inside a transaction on the underlying driver.
    pub fn transaction<T>(&self, work: impl FnOnce(&mut DbDriver) -> Result<T>) -> Result<T> {
        let mut captured_error = None;

        self.execute(|driver| {
            driver
                .transaction::<T, diesel::result::Error, _>(|driver| {
                    work(driver).map_err(|error| {
                        captured_error = Some(error);
                        diesel::result::Error::RollbackTransaction
                    })
                })
                .map_err(|error| match captured_error.take() {
                    Some(error) => error,
                    None => anyhow!(error),
                })
        })
    }

    /// Runs an async future to completion using Bevy task utilities.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        block_on(future)
    }

    fn lock_driver(&self) -> Result<MutexGuard<'_, DbDriver>> {
        self.driver
            .lock()
            .map_err(|_| anyhow!("database connection mutex was poisoned"))
    }
}

fn ensure_sqlite_parent_dir(settings: &DbSettings) -> Result<()> {
    let Some(path) = settings.sqlite_path() else {
        return Ok(());
    };
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent == Path::new("") {
        return Ok(());
    }
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "Failed to create SQLite database directory '{}'",
            parent.display()
        )
    })?;
    Ok(())
}

#[cfg(feature = "sqlite")]
fn sqlite_connection_target(database_url: &str) -> Result<String> {
    if matches!(database_url, "sqlite::memory:" | "sqlite://:memory:") {
        return Ok(":memory:".to_string());
    }

    let target = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let target = target
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(target);

    anyhow::ensure!(
        !target.trim().is_empty(),
        "SQLite database URL must include a file path or :memory:"
    );

    Ok(target.to_string())
}

#[cfg(feature = "sqlite")]
fn configure_sqlite_driver(driver: &mut DbDriver, settings: &DbSettings) -> Result<()> {
    #[allow(irrefutable_let_patterns)]
    let DbDriver::Sqlite(connection) = driver else {
        return Ok(());
    };

    let busy_timeout_ms = settings.sqlite_busy_timeout().as_millis();
    sql_query(format!("PRAGMA busy_timeout = {busy_timeout_ms}"))
        .execute(connection)
        .context("Failed to configure PRAGMA busy_timeout")?;

    let foreign_keys = if settings.sqlite_foreign_keys() {
        "ON"
    } else {
        "OFF"
    };
    sql_query(format!("PRAGMA foreign_keys = {foreign_keys}"))
        .execute(connection)
        .context("Failed to configure PRAGMA foreign_keys")?;

    if settings.sqlite_enable_wal() && !settings.is_in_memory_sqlite() {
        sql_query("PRAGMA journal_mode = WAL")
            .execute(connection)
            .context("Failed to configure PRAGMA journal_mode")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{DbSettings, DbSettingsBuilder};
    use diesel::sql_types::Integer;

    #[derive(diesel::QueryableByName)]
    struct ScalarRow {
        #[diesel(sql_type = Integer)]
        value: i32,
    }

    fn in_memory_settings() -> DbSettings {
        DbSettingsBuilder {
            database_url: "sqlite::memory:".to_string(),
            ..DbSettingsBuilder::default()
        }
        .build()
        .expect("builder should create in-memory sqlite settings")
    }

    #[test]
    fn should_detect_supported_backends_from_urls() {
        #[cfg(feature = "sqlite")]
        assert_eq!(
            DbBackend::from_url("sqlite://suon.db").expect("sqlite should be supported"),
            DbBackend::Sqlite
        );

        #[cfg(feature = "postgres")]
        assert_eq!(
            DbBackend::from_url("postgres://localhost/suon").expect("postgres should be supported"),
            DbBackend::Postgres
        );

        #[cfg(feature = "mysql")]
        {
            assert_eq!(
                DbBackend::from_url("mysql://localhost/suon").expect("mysql should be supported"),
                DbBackend::MySql
            );

            assert_eq!(
                DbBackend::from_url("mariadb://localhost/suon")
                    .expect("mariadb should be supported"),
                DbBackend::MariaDb
            );
        }
    }

    #[test]
    fn should_open_connection_from_default_sqlite_settings() {
        let connection = DbConnection::open(&in_memory_settings())
            .expect("opening an in-memory sqlite connection should succeed");

        connection
            .execute(|driver| {
                assert_eq!(driver.backend(), DbBackend::Sqlite);
                Ok(())
            })
            .expect("execute should expose the active backend");
    }

    #[test]
    fn should_run_select_queries_on_the_default_sqlite_driver() {
        let connection =
            DbConnection::open(&in_memory_settings()).expect("connection should open");

        let row = connection
            .execute(|driver| {
                sql_query("SELECT 1 AS value")
                    .get_result::<ScalarRow>(driver)
                    .context("select 1 should succeed")
            })
            .expect("default sqlite driver should run queries");

        assert_eq!(row.value, 1);
    }

    #[test]
    fn should_run_futures_through_block_on() {
        let connection =
            DbConnection::open(&in_memory_settings()).expect("connection should open");

        let value = connection.block_on(async { 42 });
        assert_eq!(value, 42);
    }

    #[test]
    fn should_clone_connection_to_share_driver() {
        let connection =
            DbConnection::open(&in_memory_settings()).expect("connection should open");
        let cloned = connection.clone();

        connection
            .execute(|driver| {
                sql_query("CREATE TABLE shared (id INTEGER PRIMARY KEY)")
                    .execute(driver)
                    .map(|_| ())
                    .map_err(anyhow::Error::from)
            })
            .expect("creating a table on the original handle should succeed");

        cloned
            .execute(|driver| {
                sql_query("INSERT INTO shared VALUES (1)")
                    .execute(driver)
                    .map(|_| ())
                    .map_err(anyhow::Error::from)
            })
            .expect("the cloned handle should observe schema from the original handle");
    }

    #[test]
    fn should_rollback_failed_transactions() {
        let connection =
            DbConnection::open(&in_memory_settings()).expect("connection should open");

        connection
            .execute(|driver| {
                sql_query("CREATE TABLE log (id INTEGER PRIMARY KEY)")
                    .execute(driver)
                    .map(|_| ())
                    .map_err(anyhow::Error::from)
            })
            .expect("schema setup should succeed");

        let result: Result<()> = connection.transaction(|driver| {
            sql_query("INSERT INTO log VALUES (1)").execute(driver)?;
            anyhow::bail!("rollback");
        });

        assert!(result.is_err());
        let count = connection
            .execute(|driver| {
                sql_query("SELECT COUNT(*) AS value FROM log")
                    .get_result::<ScalarRow>(driver)
                    .map_err(anyhow::Error::from)
            })
            .expect("counting log rows should succeed");
        assert_eq!(
            count.value, 0,
            "rolled-back transactions should not retain inserted rows"
        );
    }
}
