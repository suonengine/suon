//! Configuration models for market persistence and policy controls.
//!
//! These settings are rendered through `DocumentedToml`, so the doc comments
//! here become the inline help text users see in `MarketSettings.toml`.

use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::{Duration, SystemTime},
};
use suon_database::prelude::*;
use suon_serde::{DocumentedToml, prelude::*};

use crate::offer::MarketRateLimiter;

/// Settings that control market persistence, limits, and safety rules.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq, Default)]
pub struct MarketSettings {
    /// Persistence provider used by the market crate.
    persistence: MarketPersistenceSettings,

    /// Rules that restrict who can create offers and how often.
    policy: MarketPolicySettings,
}

impl MarketSettings {
    /// Default path used to load the market settings resource.
    pub const PATH: &'static str = "settings/MarketSettings.toml";

    /// Creates a new market settings resource.
    pub fn new(persistence: MarketPersistenceSettings, policy: MarketPolicySettings) -> Self {
        Self {
            persistence,
            policy,
        }
    }

    /// Returns the persistence settings resource.
    pub fn persistence(&self) -> &MarketPersistenceSettings {
        &self.persistence
    }

    /// Returns the market-policy settings resource.
    pub fn policy(&self) -> &MarketPolicySettings {
        &self.policy
    }

    /// Loads settings from [`Self::PATH`], creating defaults when the file does not exist.
    pub fn load_or_default() -> anyhow::Result<Self> {
        Self::load_or_default_at(Path::new(Self::PATH))
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
        debug!("Attempting to read configuration from '{}'", path.display());
        let config = fs::read_to_string(path).context("Failed to read market settings file")?;
        let settings =
            toml::from_str(&config).context("Failed to parse market settings as TOML")?;
        trace!("Loaded market settings: {:?}", settings);
        Ok(settings)
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        let default_config = Self::default();
        Self::write_at(path, &default_config)?;
        Self::load_at(path)
    }

    /// Writes the documented market settings file to disk.
    fn write_at(path: &Path, settings: &Self) -> anyhow::Result<()> {
        let config = write_documented_toml(settings)
            .context("Failed to serialize default market settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        let mut file = File::create(path).context("Failed to create the market settings file")?;
        file.write_all(config.as_bytes())
            .context("Failed to write the market settings file")?;
        file.sync_all()
            .context("Failed to flush the market settings file")?;

        Ok(())
    }

    /// Returns a log-safe summary of market persistence and policy settings.
    pub fn summary(&self) -> String {
        format!(
            "flush_interval_secs={}, save_on_shutdown={}, database_override={}, \
             max_active_offers_per_actor={}, create_rules={}, blocked_item_ids={}, \
             blocked_actor_ids={}",
            self.persistence.flush_interval().as_secs(),
            self.persistence.save_on_shutdown,
            self.persistence.database.is_some(),
            self.policy.max_active_offers_per_actor,
            self.policy.create_offer_rules.len(),
            self.policy.blocked_item_ids.len(),
            self.policy.blocked_actor_ids.len()
        )
    }
}

/// Persistence-related settings for the market crate.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq)]
pub struct MarketPersistenceSettings {
    /// How often market tables should be flushed to persistence.
    #[serde(rename = "flush_interval_secs", with = "as_secs")]
    flush_interval: Duration,

    /// Whether dirty market data should be persisted when the app exits.
    save_on_shutdown: bool,

    /// Optional database override for the market module.
    /// When omitted, the shared `DatabaseSettings` resource is used.
    database: Option<DatabaseSettings>,
}

impl Default for MarketPersistenceSettings {
    fn default() -> Self {
        Self {
            flush_interval: Duration::from_secs(1),
            save_on_shutdown: true,
            database: None,
        }
    }
}

impl MarketPersistenceSettings {
    /// Creates a new persistence settings snapshot.
    pub fn new(
        flush_interval: Duration,
        save_on_shutdown: bool,
        database: Option<DatabaseSettings>,
    ) -> Self {
        Self {
            flush_interval,
            save_on_shutdown,
            database,
        }
    }

    /// Returns the flush interval.
    pub fn flush_interval(&self) -> Duration {
        self.flush_interval
    }

    /// Returns whether dirty market data should be flushed on app exit.
    pub fn save_on_shutdown(&self) -> bool {
        self.save_on_shutdown
    }

    /// Returns the configured database override, if one exists.
    pub fn database_override(&self) -> Option<&DatabaseSettings> {
        self.database.as_ref()
    }
}

/// Policy limits that constrain market usage.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq, Eq)]
pub struct MarketPolicySettings {
    /// Maximum number of active offers allowed for a single actor.
    max_active_offers_per_actor: usize,

    /// Dynamic offer creation rules checked for each actor.
    create_offer_rules: Vec<MarketOfferCreateRule>,

    /// Item identifiers blocked from market offer creation.
    blocked_item_ids: Vec<u16>,

    /// Actor identifiers blocked from market offer creation.
    blocked_actor_ids: Vec<u32>,
}

impl Default for MarketPolicySettings {
    fn default() -> Self {
        Self {
            max_active_offers_per_actor: 100,
            create_offer_rules: vec![
                MarketOfferCreateRule::new(Duration::from_secs(60), 20),
                MarketOfferCreateRule::new(Duration::from_secs(60 * 60), 200),
            ],
            blocked_item_ids: Vec::new(),
            blocked_actor_ids: Vec::new(),
        }
    }
}

impl MarketPolicySettings {
    /// Creates a new market-policy settings snapshot.
    pub fn new(
        max_active_offers_per_actor: usize,
        create_offer_rules: Vec<MarketOfferCreateRule>,
        blocked_item_ids: Vec<u16>,
        blocked_actor_ids: Vec<u32>,
    ) -> Self {
        Self {
            max_active_offers_per_actor,
            create_offer_rules,
            blocked_item_ids,
            blocked_actor_ids,
        }
    }

    /// Returns the maximum number of active offers allowed for a single actor.
    pub fn max_active_offers_per_actor(&self) -> usize {
        self.max_active_offers_per_actor
    }

    /// Returns the configured offer creation rules.
    pub fn create_offer_rules(&self) -> &[MarketOfferCreateRule] {
        &self.create_offer_rules
    }

    /// Returns the blocked item identifiers.
    pub fn blocked_item_ids(&self) -> &[u16] {
        &self.blocked_item_ids
    }

    /// Returns the blocked actor identifiers.
    pub fn blocked_actor_ids(&self) -> &[u32] {
        &self.blocked_actor_ids
    }

    pub(crate) fn validate_offer_creation(
        &self,
        actor_id: u32,
        item_id: u16,
        active_offers: usize,
        rate_limiter: &mut MarketRateLimiter,
        now: SystemTime,
    ) -> Result<(), &'static str> {
        // Keep the checks ordered from static deny-lists to dynamic limits so
        // callers get the most direct rejection reason first.
        if self.blocked_actor_ids.contains(&actor_id) {
            return Err("actor is blocked from market offers");
        }

        if self.blocked_item_ids.contains(&item_id) {
            return Err("item is blocked from market offers");
        }

        if active_offers >= self.max_active_offers_per_actor {
            return Err("active market offer limit reached");
        }

        if !rate_limiter.record_offer_create(actor_id, now, &self.create_offer_rules) {
            return Err("market offer rate limit reached");
        }

        Ok(())
    }
}

/// Rule that limits how many offers an actor can create within a time window.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Debug, PartialEq, Eq)]
pub struct MarketOfferCreateRule {
    /// Duration of the rolling window used by this rule.
    #[serde(rename = "window_secs", with = "as_secs")]
    window: Duration,

    /// Maximum number of creates allowed within the configured window.
    max_creates: usize,
}

impl MarketOfferCreateRule {
    /// Creates a new offer creation rule.
    pub fn new(window: Duration, max_creates: usize) -> Self {
        Self {
            window,
            max_creates,
        }
    }

    /// Returns the rolling window used by this rule.
    pub fn window(&self) -> Duration {
        self.window
    }

    /// Returns the maximum number of creates allowed within the window.
    pub fn max_creates(&self) -> usize {
        self.max_creates
    }
}

impl Default for MarketOfferCreateRule {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), 20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();

        std::env::temp_dir().join(format!("{prefix}-{nanos}.toml"))
    }

    #[test]
    fn settings_roundtrip_through_toml() {
        let settings = MarketSettings::default();
        let serialized =
            toml::to_string(&settings).expect("Default market settings should serialize to TOML");
        let deserialized: MarketSettings =
            toml::from_str(&serialized).expect("Serialized settings should parse back");

        assert_eq!(deserialized, settings);
    }

    #[test]
    fn load_or_default_should_create_market_settings_when_missing() {
        let path = unique_temp_path("suon-market-settings-create");
        if path.exists() {
            fs::remove_file(&path).expect("The temp settings file should be removed");
        }

        let settings = MarketSettings::load_or_default_at(&path)
            .expect("load_or_default_at should create market settings");

        assert!(path.exists());
        assert_eq!(settings, MarketSettings::default());

        let written = fs::read_to_string(&path).expect("the market settings file should exist");
        assert!(
            written
                .contains("# Settings that control market persistence, limits, and safety rules.")
        );
        assert!(written.contains("[[policy.create_offer_rules]]"));

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }

    #[test]
    fn policy_should_default_to_shared_database_settings() {
        let settings = MarketSettings::default();
        assert!(settings.persistence().database_override().is_none());
    }
}
