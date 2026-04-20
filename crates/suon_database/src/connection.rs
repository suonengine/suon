//! Backend-neutral database connections and the default SQL-backed payload.

use crate::settings::{DatabaseConnectOptions, DatabaseSettings};
use anyhow::{Context, Result};
use bevy::tasks::block_on;
use sqlx::{AnyPool, any::install_default_drivers};

/// Marker trait for backend-specific connection data stored inside a [`DatabaseConnection`].
pub trait DatabaseData: Send + Sync + 'static {}

/// Trait for backends that expose a pool-like handle.
pub trait PoolData: DatabaseData {
    /// Concrete pool type exposed by the backend payload.
    type Pool;

    /// Returns the pool-like handle carried by the backend payload.
    fn pool(&self) -> &Self::Pool;
}

/// Backend-specific connection data plus task-backed async helpers.
pub struct DatabaseConnection<D: DatabaseData> {
    /// Backend-specific state, such as a SQL pool.
    data: D,
}

impl<D: DatabaseData> DatabaseConnection<D> {
    /// Builds a connection wrapper from backend-specific payload.
    pub fn new(data: D) -> Self {
        Self { data }
    }

    /// Runs an async future to completion using Bevy task utilities.
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        block_on(future)
    }

    /// Returns the backend-specific payload.
    pub fn data(&self) -> &D {
        &self.data
    }
}

/// `AnyPool` storage used by the default persistence integration.
pub struct DatabasePool {
    /// Lazy pool configured from [`DatabaseSettings`].
    pool: AnyPool,
}

impl DatabaseData for DatabasePool {}

impl PoolData for DatabasePool {
    type Pool = AnyPool;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }
}

impl DatabaseConnection<DatabasePool> {
    /// Builds a SQL-backed connection using Bevy task utilities for async execution.
    pub fn connect(settings: &DatabaseSettings) -> Result<Self> {
        let options = DatabaseConnectOptions::from_settings(settings)?;
        install_default_drivers();

        let pool = options
            .pool_options
            .connect_lazy(&options.database_url)
            .with_context(|| {
                format!(
                    "Failed to create lazy database pool for URL '{}'",
                    options.database_url
                )
            })?;

        Ok(Self::new(DatabasePool { pool }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::DatabaseSettingsBuilder;
    use std::sync::{Arc, Mutex};

    #[test]
    fn should_build_database_connection_from_shared_runtime() {
        let database = DatabaseConnection::<DatabasePool>::connect(&DatabaseSettings::default())
            .expect("database connection should build");

        let _ = database.data().pool();
    }

    #[test]
    fn should_build_sql_connection_with_custom_pool_options() {
        let settings = DatabaseSettingsBuilder {
            min_connections: 0,
            max_connections: 8,
            acquire_timeout_secs: 5,
            idle_timeout_secs: Some(60),
            max_lifetime_secs: Some(600),
            test_before_acquire: false,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("builder should create valid custom pool settings");

        let database = DatabaseConnection::<DatabasePool>::connect(&settings)
            .expect("database connection should build with custom pool options");

        let _ = database.data().pool();
    }

    #[test]
    fn should_expose_backend_data_from_connection() {
        struct DemoData {
            value: usize,
        }

        impl DatabaseData for DemoData {}

        let connection = DatabaseConnection::new(DemoData { value: 11 });
        assert_eq!(
            connection.data().value,
            11,
            "DatabaseConnection::data should expose the backend-specific payload stored inside \
             the connection"
        );
    }

    #[test]
    fn should_run_futures_through_connection_block_on() {
        struct DemoData {
            state: Arc<Mutex<Vec<&'static str>>>,
        }

        impl DatabaseData for DemoData {}

        let state = Arc::new(Mutex::new(Vec::new()));
        let connection = DatabaseConnection::new(DemoData {
            state: state.clone(),
        });

        connection.block_on(async {
            state
                .lock()
                .expect("state mutex should stay available")
                .push("ran");
        });

        assert_eq!(
            connection
                .data()
                .state
                .lock()
                .expect("state mutex should stay available")
                .as_slice(),
            &["ran"],
            "DatabaseConnection::block_on should drive async work to completion through Bevy task \
             utilities"
        );
    }

    #[test]
    fn should_report_invalid_database_urls_when_connecting() {
        let settings = DatabaseSettingsBuilder {
            database_url: "://not-a-valid-database-url".to_string(),
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("the builder should accept non-empty URLs and let connect validate them");

        let error = DatabaseConnection::<DatabasePool>::connect(&settings)
            .err()
            .expect("connect should reject malformed database URLs");

        assert!(
            error
                .to_string()
                .contains("Failed to create lazy database pool for URL"),
            "connect should attach the failing database URL to connection errors"
        );
    }
}
