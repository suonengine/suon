//! Generic settings shared by Diesel-backed persistence providers.

use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use suon_serde::{DocumentedToml, prelude::*};

/// Generic persistence settings used by Diesel-backed providers.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq)]
pub struct DatabaseSettings {
    /// Database connection URL.
    /// Supported backends: SQLite, PostgreSQL, MySQL, and MariaDB.
    database_url: String,

    /// Busy timeout applied to SQLite connections.
    #[serde(rename = "sqlite_busy_timeout_ms", with = "as_millis")]
    sqlite_busy_timeout: Duration,

    /// Enables foreign-key enforcement on SQLite.
    sqlite_foreign_keys: bool,

    /// Enables write-ahead logging on file-based SQLite databases.
    sqlite_enable_wal: bool,

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

    /// Database URL used by Diesel.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Database URL normalized for the target backend.
    pub fn normalized_database_url(&self) -> String {
        normalize_database_url(&self.database_url)
    }

    /// Busy timeout applied to SQLite connections.
    pub fn sqlite_busy_timeout(&self) -> Duration {
        self.sqlite_busy_timeout
    }

    /// Whether SQLite foreign keys should be enabled.
    pub fn sqlite_foreign_keys(&self) -> bool {
        self.sqlite_foreign_keys
    }

    /// Whether write-ahead logging should be enabled for file-based SQLite databases.
    pub fn sqlite_enable_wal(&self) -> bool {
        self.sqlite_enable_wal
    }

    /// Whether mappers should initialize schema during startup.
    pub fn auto_initialize_schema(&self) -> bool {
        self.auto_initialize_schema
    }

    /// Returns the filesystem path when the URL targets a SQLite file database.
    pub fn sqlite_path(&self) -> Option<PathBuf> {
        sqlite_path_from_url(&self.database_url)
    }

    /// Returns whether the URL targets an in-memory SQLite database.
    pub fn is_in_memory_sqlite(&self) -> bool {
        matches!(
            parse_database_target(&self.database_url),
            DatabaseTarget::SqliteMemory
        )
    }

    /// Returns a log-safe summary of the database settings.
    pub fn summary(&self) -> String {
        format!(
            "backend={}, target={}, sqlite_busy_timeout_ms={}, sqlite_foreign_keys={}, \
             sqlite_enable_wal={}, auto_initialize_schema={}",
            backend_name(&self.database_url),
            self.redacted_target(),
            self.sqlite_busy_timeout.as_millis(),
            self.sqlite_foreign_keys,
            self.sqlite_enable_wal,
            self.auto_initialize_schema
        )
    }

    fn redacted_target(&self) -> String {
        match parse_database_target(&self.database_url) {
            DatabaseTarget::SqliteMemory => ":memory:".to_string(),
            DatabaseTarget::SqliteFile(path) => path
                .file_name()
                .and_then(|name| name.to_str())
                .map_or_else(|| "<sqlite-file>".to_string(), ToString::to_string),
            DatabaseTarget::Server => "<redacted>".to_string(),
            DatabaseTarget::Invalid => "<invalid>".to_string(),
        }
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

    pub(crate) fn load_at(path: &Path) -> anyhow::Result<Self> {
        let config = fs::read_to_string(path).context("Failed to read database settings file")?;
        let settings: Self =
            toml::from_str(&config).context("Failed to parse database settings as TOML")?;
        DatabaseSettingsBuilder::from(&settings).build()
    }

    pub(crate) fn create_at(path: &Path) -> anyhow::Result<Self> {
        let default_config = Self::default();
        Self::write_at(path, &default_config)?;
        Self::load_at(path)
    }

    fn write_at(path: &Path, settings: &Self) -> anyhow::Result<()> {
        let config = write_documented_toml(settings)
            .context("Failed to serialize default database settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        let mut file = File::create(path).context("Failed to create the database settings file")?;
        file.write_all(config.as_bytes())
            .context("Failed to write the database settings file")?;
        file.sync_all()
            .context("Failed to flush the database settings file")?;

        Ok(())
    }
}

/// Parsed target classification used to produce log-safe setting summaries.
enum DatabaseTarget {
    SqliteMemory,
    SqliteFile(PathBuf),
    Server,
    Invalid,
}

/// Classifies a database URL into a coarse backend target category.
fn parse_database_target(database_url: &str) -> DatabaseTarget {
    if matches!(database_url, "sqlite::memory:" | "sqlite://:memory:") {
        return DatabaseTarget::SqliteMemory;
    }

    if let Some(path) = sqlite_path_from_url(database_url) {
        return DatabaseTarget::SqliteFile(path);
    }

    if matches_scheme(
        database_url,
        &["postgres://", "postgresql://", "mysql://", "mariadb://"],
    ) {
        return DatabaseTarget::Server;
    }

    DatabaseTarget::Invalid
}

/// Returns whether the URL begins with any of the provided schemes.
fn matches_scheme(database_url: &str, schemes: &[&str]) -> bool {
    schemes
        .iter()
        .any(|scheme| database_url.starts_with(scheme))
}

/// Returns a short backend label safe to expose in logs.
fn backend_name(database_url: &str) -> &str {
    if database_url.starts_with("sqlite:") {
        "sqlite"
    } else if matches_scheme(database_url, &["postgres://", "postgresql://"]) {
        "postgres"
    } else if matches_scheme(database_url, &["mysql://", "mariadb://"]) {
        "mysql"
    } else {
        "unknown"
    }
}

/// Normalizes backend aliases into the form expected by Diesel clients.
fn normalize_database_url(database_url: &str) -> String {
    if let Some(stripped) = database_url.strip_prefix("mariadb://") {
        return format!("mysql://{stripped}");
    }

    database_url.to_string()
}

/// Extracts the backing filesystem path from a SQLite URL when present.
fn sqlite_path_from_url(database_url: &str) -> Option<PathBuf> {
    if !database_url.starts_with("sqlite:") {
        return None;
    }

    let target = database_url
        .trim_start_matches("sqlite://")
        .trim_start_matches("sqlite:");
    let target = target
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(target);

    match target {
        "" | ":memory:" => None,
        value => Some(PathBuf::from(value)),
    }
}

/// Public builder used to create validated [`DatabaseSettings`] values.
#[derive(Clone, Debug, PartialEq)]
pub struct DatabaseSettingsBuilder {
    /// Database URL.
    pub database_url: String,

    /// Busy timeout applied to SQLite connections.
    pub sqlite_busy_timeout: Duration,

    /// Whether SQLite foreign keys should be enabled.
    pub sqlite_foreign_keys: bool,

    /// Whether write-ahead logging should be enabled for file-based SQLite databases.
    pub sqlite_enable_wal: bool,

    /// Whether mappers should initialize schema during startup.
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
            matches_scheme(
                &self.database_url,
                &[
                    "sqlite:",
                    "postgres://",
                    "postgresql://",
                    "mysql://",
                    "mariadb://"
                ]
            ),
            "DatabaseSettings.database_url must use sqlite:, postgres://, postgresql://, \
             mysql://, or mariadb://"
        );
        anyhow::ensure!(
            self.sqlite_busy_timeout > Duration::ZERO,
            "DatabaseSettings.sqlite_busy_timeout must be greater than zero"
        );

        Ok(DatabaseSettings {
            database_url: self.database_url,
            sqlite_busy_timeout: self.sqlite_busy_timeout,
            sqlite_foreign_keys: self.sqlite_foreign_keys,
            sqlite_enable_wal: self.sqlite_enable_wal,
            auto_initialize_schema: self.auto_initialize_schema,
        })
    }
}

impl From<&DatabaseSettings> for DatabaseSettingsBuilder {
    fn from(settings: &DatabaseSettings) -> Self {
        Self {
            database_url: settings.database_url.clone(),
            sqlite_busy_timeout: settings.sqlite_busy_timeout,
            sqlite_foreign_keys: settings.sqlite_foreign_keys,
            sqlite_enable_wal: settings.sqlite_enable_wal,
            auto_initialize_schema: settings.auto_initialize_schema,
        }
    }
}

impl Default for DatabaseSettingsBuilder {
    fn default() -> Self {
        let settings = DatabaseSettings::default();

        Self {
            database_url: settings.database_url,
            sqlite_busy_timeout: settings.sqlite_busy_timeout,
            sqlite_foreign_keys: settings.sqlite_foreign_keys,
            sqlite_enable_wal: settings.sqlite_enable_wal,
            auto_initialize_schema: settings.auto_initialize_schema,
        }
    }
}

impl Default for DatabaseSettings {
    fn default() -> Self {
        Self {
            database_url: "sqlite://suon.db".to_string(),
            sqlite_busy_timeout: Duration::from_secs(30),
            sqlite_foreign_keys: true,
            sqlite_enable_wal: true,
            auto_initialize_schema: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseSettings, DatabaseSettingsBuilder};
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

        assert_eq!(deserialized, settings);
    }

    #[test]
    fn should_provide_predictable_default_database_settings() {
        let settings = DatabaseSettings::default();

        assert_eq!(settings.database_url(), "sqlite://suon.db");
        assert_eq!(settings.sqlite_busy_timeout(), Duration::from_secs(30));
        assert!(settings.sqlite_foreign_keys());
        assert!(settings.sqlite_enable_wal());
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
    fn should_parse_supported_database_urls() {
        assert!(
            DatabaseSettingsBuilder {
                database_url: "sqlite::memory:".to_string(),
                ..DatabaseSettingsBuilder::default()
            }
            .build()
            .expect("memory database should be valid")
            .is_in_memory_sqlite()
        );

        assert_eq!(
            DatabaseSettingsBuilder {
                database_url: "mariadb://user:secret@localhost/suon".to_string(),
                ..DatabaseSettingsBuilder::default()
            }
            .build()
            .expect("mariadb should be valid")
            .normalized_database_url(),
            "mysql://user:secret@localhost/suon"
        );
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
    fn validate_should_reject_unsupported_urls() {
        let error = DatabaseSettingsBuilder {
            database_url: "sqlserver://localhost/suon".to_string(),
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject unsupported urls");

        assert!(error.to_string().contains(
            "DatabaseSettings.database_url must use sqlite:, postgres://, postgresql://, \
             mysql://, or mariadb://"
        ));
    }

    #[test]
    fn validate_should_reject_zero_busy_timeout() {
        let error = DatabaseSettingsBuilder {
            sqlite_busy_timeout: Duration::ZERO,
            ..DatabaseSettingsBuilder::default()
        }
        .build()
        .expect_err("validate should reject zero busy timeouts");

        assert!(
            error
                .to_string()
                .contains("DatabaseSettings.sqlite_busy_timeout must be greater than zero")
        );
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
        assert!(written.contains("sqlite_busy_timeout_ms = 30000"));
    }

    #[test]
    fn should_load_existing_settings_from_disk() {
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp settings directory should be created");

        let config_path = temp_dir.join("Settings.toml");
        let config = r#"
database_url = "postgres://loaded"
sqlite_busy_timeout_ms = 1500
sqlite_foreign_keys = false
sqlite_enable_wal = false
auto_initialize_schema = false
"#;

        fs::write(&config_path, config).expect("the test configuration file should be written");

        let loaded = DatabaseSettings::load_at(&config_path)
            .expect("load_at should deserialize existing TOML settings");

        assert_eq!(loaded.database_url(), "postgres://loaded");
        assert_eq!(loaded.sqlite_busy_timeout(), Duration::from_millis(1500));
        assert!(!loaded.sqlite_foreign_keys());
        assert!(!loaded.sqlite_enable_wal());
        assert!(!loaded.auto_initialize_schema());
    }
}
