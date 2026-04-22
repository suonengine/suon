//! Generic settings shared by database-backed persistence providers.

use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::any::AnyPoolOptions;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::Duration,
};
use suon_serde::{DocumentedToml, prelude::*};

/// Generic persistence settings used by database-backed providers.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq)]
pub struct DatabaseSettings {
    /// Database connection URL.
    /// Supported backends: SQLite, PostgreSQL, MySQL, and MariaDB.
    /// Examples: `sqlite://suon.db`, `postgres://user:password@localhost/suon`,
    /// `mysql://user:password@localhost/suon`, `mariadb://user:password@localhost/suon`
    database_url: String,

    /// Minimum number of connections kept ready by the backing pool.
    min_connections: u32,

    /// Maximum number of concurrent connections allowed by the backing pool.
    max_connections: u32,

    /// Maximum time spent waiting to acquire a connection from the pool.
    #[serde(rename = "acquire_timeout_secs", with = "as_secs")]
    acquire_timeout: Duration,

    /// Maximum idle time before an unused connection is recycled.
    #[serde(default, rename = "idle_timeout_secs", with = "as_secs::option")]
    idle_timeout: Option<Duration>,

    /// Maximum lifetime before a connection is recycled.
    #[serde(default, rename = "max_lifetime_secs", with = "as_secs::option")]
    max_lifetime: Option<Duration>,

    /// Whether the pool should validate a connection before handing it out.
    test_before_acquire: bool,

    /// Whether mappers should initialize the schema during startup.
    auto_initialize_schema: bool,
}

impl DatabaseSettings {
    /// Path to the configuration file.
    pub const PATH: &'static str = "settings/DatabaseSettings.toml";

    /// Loads the settings file or creates a default one when it is missing.
    pub fn load_or_default() -> anyhow::Result<Self> {
        Self::load_or_default_at(Path::new(Self::PATH))
    }

    /// Creates a builder initialized with the default database settings.
    pub fn builder() -> DatabaseSettingsBuilder {
        DatabaseSettingsBuilder::default()
    }

    /// URL used by the backing database driver, for example `sqlite://market.db`.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Minimum number of connections kept ready by the backing pool.
    pub fn min_connections(&self) -> u32 {
        self.min_connections
    }

    /// Maximum number of concurrent connections allowed by the backing pool.
    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Maximum time spent waiting to acquire a connection from the pool.
    pub fn acquire_timeout(&self) -> Duration {
        self.acquire_timeout
    }

    /// Maximum idle time before an unused connection is recycled.
    pub fn idle_timeout(&self) -> Option<Duration> {
        self.idle_timeout
    }

    /// Maximum lifetime before a connection is recycled.
    pub fn max_lifetime(&self) -> Option<Duration> {
        self.max_lifetime
    }

    /// Whether the pool should validate a connection before handing it out.
    pub fn test_before_acquire(&self) -> bool {
        self.test_before_acquire
    }

    /// Whether mappers should initialize the schema during startup.
    pub fn auto_initialize_schema(&self) -> bool {
        self.auto_initialize_schema
    }

    /// Returns a log-safe summary of the database settings.
    pub fn summary(&self) -> String {
        format!(
            "backend={}, target={}, pool_min={}, pool_max={}, acquire_timeout_secs={}, \
             idle_timeout_secs={}, max_lifetime_secs={}, test_before_acquire={}, \
             auto_initialize_schema={}",
            self.backend_name(),
            self.redacted_target(),
            self.min_connections,
            self.max_connections,
            self.acquire_timeout.as_secs(),
            option_or_disabled(self.idle_timeout),
            option_or_disabled(self.max_lifetime),
            self.test_before_acquire,
            self.auto_initialize_schema
        )
    }

    fn backend_name(&self) -> &str {
        self.database_url
            .split_once(':')
            .map(|(scheme, _)| scheme)
            .unwrap_or("unknown")
    }

    fn redacted_target(&self) -> String {
        if self.database_url.starts_with("sqlite:") {
            return redact_sqlite_target(&self.database_url);
        }

        "<redacted>".to_string()
    }

    fn load_or_default_at(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            info!(
                "Configuration file '{}' found, attempting to load.",
                path.display()
            );

            return Self::load_at(path);
        }

        warn!(
            "Configuration file '{}' not found. Creating default configuration.",
            path.display()
        );

        Self::create_at(path)
    }

    fn load_at(path: &Path) -> anyhow::Result<Self> {
        debug!(
            "Attempting to read database configuration from '{}'",
            path.display()
        );

        let config = fs::read_to_string(path).context("Failed to read database settings file")?;

        info!(
            "Successfully read database configuration file '{}'",
            path.display()
        );

        let settings: Self =
            toml::from_str(&config).context("Failed to parse database settings as TOML")?;

        let settings = DatabaseSettingsBuilder::from(&settings).build()?;

        trace!("Loaded database settings: {:?}", settings);

        Ok(settings)
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        let default_config = Self::default();

        info!(
            "Creating default database configuration file '{}'",
            path.display()
        );

        Self::write_at(path, &default_config)?;

        info!(
            "Default database configuration written to '{}'. Reloading from file.",
            path.display()
        );

        Self::load_at(path)
    }

    fn write_at(path: &Path, settings: &Self) -> anyhow::Result<()> {
        debug!("Rendering documented database configuration");

        let config = write_documented_toml(settings)
            .context("Failed to serialize default database settings")?;

        if let Some(parent) = path.parent() {
            debug!(
                "Ensuring database settings directory '{}' exists",
                parent.display()
            );
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        debug!(
            "Creating database configuration file at '{}'",
            path.display()
        );

        let mut file = File::create(path).context("Failed to create the database settings file")?;

        debug!("Writing database configuration to file");

        file.write_all(config.as_bytes())
            .context("Failed to write the database settings file")?;

        file.sync_all()
            .context("Failed to flush the database settings file")?;

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct DatabaseConnectOptions {
    pub(crate) database_url: String,
    pub(crate) pool_options: AnyPoolOptions,
}

impl DatabaseConnectOptions {
    /// Creates validated connection options from raw database settings.
    pub(crate) fn from_settings(settings: &DatabaseSettings) -> anyhow::Result<Self> {
        let settings = DatabaseSettingsBuilder::from(settings).build()?;

        let pool_options = AnyPoolOptions::new()
            .min_connections(settings.min_connections)
            .max_connections(settings.max_connections)
            .acquire_timeout(settings.acquire_timeout)
            .idle_timeout(settings.idle_timeout)
            .max_lifetime(settings.max_lifetime)
            .test_before_acquire(settings.test_before_acquire);

        Ok(Self {
            database_url: database_url_with_sqlite_create_mode(&settings.database_url),
            pool_options,
        })
    }
}

fn database_url_with_sqlite_create_mode(database_url: &str) -> String {
    if !database_url.starts_with("sqlite:") {
        return database_url.to_string();
    }

    let database_and_params = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let mut parts = database_and_params.splitn(2, '?');
    let database = parts.next().unwrap_or_default();
    let params = parts.next();

    if database == ":memory:" {
        return database_url.to_string();
    }

    if params.is_some_and(|params| {
        params.split('&').any(|pair| {
            pair.split_once('=')
                .map_or(pair == "mode", |(key, _)| key == "mode")
        })
    }) {
        return database_url.to_string();
    }

    let separator = if database_url.contains('?') { '&' } else { '?' };
    format!("{database_url}{separator}mode=rwc")
}

fn redact_sqlite_target(database_url: &str) -> String {
    let database_and_params = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let database = database_and_params
        .split_once('?')
        .map(|(database, _)| database)
        .unwrap_or(database_and_params);

    if database == ":memory:" {
        return ":memory:".to_string();
    }

    if database.is_empty() {
        return "<default>".to_string();
    }

    Path::new(database)
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "<sqlite-file>".to_string(), ToString::to_string)
}

fn option_or_disabled(value: Option<Duration>) -> String {
    value.map_or_else(
        || "disabled".to_string(),
        |value| value.as_secs().to_string(),
    )
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            database_url: "sqlite://suon.db?mode=rwc".to_string(),
            min_connections: 1,
            max_connections: 4,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(300)),
            max_lifetime: Some(Duration::from_secs(1800)),
            test_before_acquire: true,
            auto_initialize_schema: true,
        }
    }
}

/// Public builder used to create validated [`DatabaseSettings`] values.
#[derive(Clone, Debug, PartialEq)]
pub struct DatabaseSettingsBuilder {
    /// Database connection URL.
    /// Supported backends: SQLite, PostgreSQL, MySQL, and MariaDB.
    /// Examples: `sqlite://suon.db`, `postgres://user:password@localhost/suon`,
    /// `mysql://user:password@localhost/suon`, `mariadb://user:password@localhost/suon`
    pub database_url: String,

    /// Minimum number of connections kept ready by the backing pool.
    pub min_connections: u32,

    /// Maximum number of concurrent connections allowed by the backing pool.
    pub max_connections: u32,

    /// Maximum time spent waiting to acquire a connection from the pool.
    pub acquire_timeout: Duration,

    /// Maximum idle time before an unused connection is recycled.
    pub idle_timeout: Option<Duration>,

    /// Maximum lifetime before a connection is recycled.
    pub max_lifetime: Option<Duration>,

    /// Whether the pool should validate a connection before handing it out.
    pub test_before_acquire: bool,

    /// Whether mappers should initialize the schema during startup.
    pub auto_initialize_schema: bool,
}

impl DatabaseSettingsBuilder {
    /// Creates a builder initialized with the default settings values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds validated database settings from the builder contents.
    pub fn build(self) -> anyhow::Result<DatabaseSettings> {
        anyhow::ensure!(
            !self.database_url.trim().is_empty(),
            "DatabaseSettings.database_url must not be empty"
        );
        anyhow::ensure!(
            self.max_connections > 0,
            "DatabaseSettings.max_connections must be greater than zero"
        );
        anyhow::ensure!(
            self.min_connections <= self.max_connections,
            "DatabaseSettings.min_connections must not exceed DatabaseSettings.max_connections"
        );
        anyhow::ensure!(
            self.acquire_timeout > Duration::ZERO,
            "DatabaseSettings.acquire_timeout must be greater than zero"
        );

        let settings = DatabaseSettings {
            database_url: self.database_url,
            min_connections: self.min_connections,
            max_connections: self.max_connections,
            acquire_timeout: self.acquire_timeout,
            idle_timeout: self.idle_timeout,
            max_lifetime: self.max_lifetime,
            test_before_acquire: self.test_before_acquire,
            auto_initialize_schema: self.auto_initialize_schema,
        };

        debug!("Database settings validated successfully.");
        Ok(settings)
    }
}

impl From<&DatabaseSettings> for DatabaseSettingsBuilder {
    fn from(settings: &DatabaseSettings) -> Self {
        Self {
            database_url: settings.database_url.clone(),
            min_connections: settings.min_connections,
            max_connections: settings.max_connections,
            acquire_timeout: settings.acquire_timeout,
            idle_timeout: settings.idle_timeout,
            max_lifetime: settings.max_lifetime,
            test_before_acquire: settings.test_before_acquire,
            auto_initialize_schema: settings.auto_initialize_schema,
        }
    }
}

impl Default for DatabaseSettingsBuilder {
    fn default() -> Self {
        let settings = DatabaseSettings::default();

        Self {
            database_url: settings.database_url,
            min_connections: settings.min_connections,
            max_connections: settings.max_connections,
            acquire_timeout: settings.acquire_timeout,
            idle_timeout: settings.idle_timeout,
            max_lifetime: settings.max_lifetime,
            test_before_acquire: settings.test_before_acquire,
            auto_initialize_schema: settings.auto_initialize_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseConnectOptions, DatabaseSettings, DatabaseSettingsBuilder};
    use std::{
        env, fs,
        path::PathBuf,
        process,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("suon-database-settings-{}-{nanos}", process::id()))
    }

    #[test]
    fn database_settings_roundtrip_through_toml() {
        let settings = DatabaseSettings::default();

        let serialized =
            toml::to_string(&settings).expect("default database settings should serialize");

        let deserialized: DatabaseSettings =
            toml::from_str(&serialized).expect("serialized settings should parse back");

        assert_eq!(
            deserialized, settings,
            "DatabaseSettings should round-trip through TOML without losing information"
        );
    }

    #[test]
    fn should_provide_predictable_default_database_settings() {
        let settings = DatabaseSettings::default();

        assert_eq!(settings.database_url(), "sqlite://suon.db?mode=rwc");
        assert_eq!(settings.max_connections(), 4);
        assert_eq!(settings.min_connections(), 1);
        assert_eq!(settings.acquire_timeout(), Duration::from_secs(30));
        assert_eq!(settings.idle_timeout(), Some(Duration::from_secs(300)));
        assert_eq!(settings.max_lifetime(), Some(Duration::from_secs(1800)));
        assert!(settings.test_before_acquire());
        assert!(settings.auto_initialize_schema());
    }

    #[test]
    fn should_expose_builder_constructors_with_default_values() {
        assert_eq!(
            DatabaseSettings::builder(),
            DatabaseSettingsBuilder::default()
        );
        assert_eq!(
            DatabaseSettingsBuilder::new(),
            DatabaseSettingsBuilder::default()
        );
    }

    #[test]
    fn should_support_clone_debug_and_partial_eq_for_settings() {
        let settings = DatabaseSettings::default();
        let cloned = settings.clone();

        assert_eq!(cloned, settings);
        assert!(format!("{cloned:?}").contains("sqlite://suon.db?mode=rwc"));
    }

    #[test]
    fn should_build_pool_options_from_timeout_settings() {
        let settings = DatabaseSettingsBuilder {
            acquire_timeout: Duration::from_secs(9),
            idle_timeout: Some(Duration::from_secs(12)),
            max_lifetime: Some(Duration::from_secs(18)),
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("builder should create valid timeout settings");

        let options = DatabaseConnectOptions::from_settings(&settings)
            .expect("from_settings should convert timeout fields into pool options");

        assert_eq!(options.database_url, "sqlite://suon.db?mode=rwc");
    }

    #[test]
    fn should_add_create_mode_to_legacy_sqlite_file_urls() {
        let settings = DatabaseSettingsBuilder {
            database_url: "sqlite://legacy.db".to_string(),
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("builder should accept legacy sqlite urls");

        let options = DatabaseConnectOptions::from_settings(&settings)
            .expect("from_settings should convert sqlite file urls into connection options");

        assert_eq!(options.database_url, "sqlite://legacy.db?mode=rwc");
    }

    #[test]
    fn should_preserve_explicit_sqlite_modes_and_memory_urls() {
        for database_url in [
            "sqlite::memory:",
            "sqlite://:memory:",
            "sqlite://readonly.db?mode=ro",
        ] {
            let settings = DatabaseSettingsBuilder {
                database_url: database_url.to_string(),
                ..DatabaseSettingsBuilder::default()
            }
            .build()
            .expect("builder should accept sqlite urls");

            let options = DatabaseConnectOptions::from_settings(&settings)
                .expect("from_settings should convert sqlite urls into connection options");

            assert_eq!(options.database_url, database_url);
        }
    }

    #[test]
    fn should_preserve_non_sqlite_any_pool_urls() {
        for database_url in [
            "postgres://suon:secret@localhost/suon",
            "mysql://suon:secret@localhost/suon",
        ] {
            let settings = DatabaseSettingsBuilder {
                database_url: database_url.to_string(),
                ..DatabaseSettingsBuilder::default()
            }
            .build()
            .expect("builder should accept non-empty AnyPool urls");

            let options = DatabaseConnectOptions::from_settings(&settings)
                .expect("from_settings should preserve non-sqlite URLs");

            assert_eq!(options.database_url, database_url);
        }
    }

    #[test]
    fn should_allow_optional_pool_timeouts_to_be_disabled() {
        let settings = DatabaseSettingsBuilder {
            idle_timeout: None,
            max_lifetime: None,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("builder should allow optional pool recycling settings to be disabled");

        let options = DatabaseConnectOptions::from_settings(&settings)
            .expect("from_settings should allow optional pool recycling settings to be disabled");

        assert!(format!("{:?}", options.pool_options).contains("PoolOptions"));
    }

    #[test]
    fn validate_should_reject_blank_database_urls() {
        let error = DatabaseSettingsBuilder {
            database_url: " ".to_string(),
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject blank database urls");

        assert!(
            error
                .to_string()
                .contains("DatabaseSettings.database_url must not be empty")
        );
    }

    #[test]
    fn validate_should_reject_zero_max_connections() {
        let error = DatabaseSettingsBuilder {
            max_connections: 0,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject zero max connections");

        assert!(
            error
                .to_string()
                .contains("DatabaseSettings.max_connections must be greater than zero")
        );
    }

    #[test]
    fn validate_should_reject_min_connections_above_max_connections() {
        let error = DatabaseSettingsBuilder {
            min_connections: 5,
            max_connections: 4,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject min connections above max connections");

        assert!(error.to_string().contains(
            "DatabaseSettings.min_connections must not exceed DatabaseSettings.max_connections"
        ));
    }

    #[test]
    fn validate_should_reject_zero_acquire_timeout() {
        let error = DatabaseSettingsBuilder {
            acquire_timeout: Duration::ZERO,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject zero acquire timeouts");

        assert!(
            error
                .to_string()
                .contains("DatabaseSettings.acquire_timeout must be greater than zero")
        );
    }

    #[test]
    fn builder_should_create_settings_from_public_fields() {
        let settings = DatabaseSettingsBuilder {
            database_url: "sqlite://builder.db".to_string(),
            max_connections: 8,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect("builder should produce validated settings");

        assert_eq!(settings.database_url(), "sqlite://builder.db");
        assert_eq!(settings.max_connections(), 8);
    }

    #[test]
    fn should_create_default_settings_file_when_missing() {
        let temp_dir = unique_temp_dir();
        let config_path = temp_dir.join("nested").join("Settings.toml");

        let created = DatabaseSettings::create_at(&config_path)
            .expect("create_at should write and then reload the default settings file");

        assert_eq!(created, DatabaseSettings::default());
        assert!(config_path.exists());

        let written =
            fs::read_to_string(&config_path).expect("the documented settings file should exist");
        assert!(
            written.contains("# Generic persistence settings used by database-backed providers.")
        );
        assert!(written.contains("acquire_timeout_secs = 30"));
    }

    #[test]
    fn should_load_existing_settings_from_disk() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp settings directory should be created");

        let config_path = temp_dir.join("Settings.toml");
        let config = r#"
database_url = "sqlite://loaded.db"
min_connections = 2
max_connections = 6
acquire_timeout_secs = 15
idle_timeout_secs = 120
max_lifetime_secs = 900
test_before_acquire = false
auto_initialize_schema = false
"#;

        fs::write(&config_path, config).expect("the test configuration file should be written");

        let loaded = DatabaseSettings::load_at(&config_path)
            .expect("load_at should deserialize existing TOML settings");

        assert_eq!(loaded.database_url(), "sqlite://loaded.db");
        assert_eq!(loaded.min_connections(), 2);
        assert_eq!(loaded.max_connections(), 6);
        assert_eq!(loaded.acquire_timeout(), Duration::from_secs(15));
        assert_eq!(loaded.idle_timeout(), Some(Duration::from_secs(120)));
        assert_eq!(loaded.max_lifetime(), Some(Duration::from_secs(900)));
        assert!(!loaded.test_before_acquire());
        assert!(!loaded.auto_initialize_schema());
    }
}
