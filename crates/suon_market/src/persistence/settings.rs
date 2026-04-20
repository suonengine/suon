use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::SystemTime,
};
use suon_database::prelude::*;

use crate::offer::MarketRateLimiter;

/// Settings that control market persistence, limits, and safety rules.
#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MarketSettings {
    /// Persistence provider used by the market crate.
    pub persistence: MarketPersistenceSettings,

    /// Rules that restrict who can create offers and how often.
    pub policy: MarketPolicySettings,
}

impl MarketSettings {
    pub const PATH: &'static str = "settings/MarketSettings.toml";

    pub fn load_or_default() -> anyhow::Result<Self> {
        Self::load_or_default_at(Path::new(Self::PATH))
    }

    fn load_or_default_at(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            info!(
                "Configuration file '{}' found, attempting to load.",
                path.display()
            );
            Self::load_at(path)
        } else {
            warn!(
                "Configuration file '{}' not found. Creating default configuration.",
                path.display()
            );
            Self::create_at(path)
        }
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
        let config = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default market settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        let mut file = File::create(path).context("Failed to create the market settings file")?;
        file.write_all(config.as_bytes())
            .context("Failed to write the default market settings file")?;
        file.sync_all()
            .context("Failed to flush the default market settings file")?;

        Self::load_at(path)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for MarketSettings {
    fn default() -> Self {
        Self {
            persistence: MarketPersistenceSettings::default(),
            policy: MarketPolicySettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MarketPersistenceSettings {
    pub flush_interval_secs: f64,
    pub save_on_shutdown: bool,
    pub database: DatabaseSettings,
}

impl Default for MarketPersistenceSettings {
    fn default() -> Self {
        Self {
            flush_interval_secs: 1.0,
            save_on_shutdown: true,
            database: DatabaseSettings::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MarketPolicySettings {
    pub max_active_offers_per_player: usize,
    pub max_create_per_minute: usize,
    pub max_create_per_hour: usize,
    pub blocked_item_ids: Vec<u16>,
    pub blocked_player_ids: Vec<u32>,
}

impl Default for MarketPolicySettings {
    fn default() -> Self {
        Self {
            max_active_offers_per_player: 100,
            max_create_per_minute: 20,
            max_create_per_hour: 200,
            blocked_item_ids: Vec::new(),
            blocked_player_ids: Vec::new(),
        }
    }
}

impl MarketPolicySettings {
    pub(crate) fn validate_offer_creation(
        &self,
        player_id: u32,
        item_id: u16,
        active_offers: usize,
        rate_limiter: &mut MarketRateLimiter,
        now: SystemTime,
    ) -> Result<(), &'static str> {
        if self.blocked_player_ids.contains(&player_id) {
            return Err("player is blocked from market offers");
        }

        if self.blocked_item_ids.contains(&item_id) {
            return Err("item is blocked from market offers");
        }

        if active_offers >= self.max_active_offers_per_player {
            return Err("active market offer limit reached");
        }

        if !rate_limiter.record_offer_create(
            player_id,
            now,
            self.max_create_per_minute,
            self.max_create_per_hour,
        ) {
            return Err("market offer rate limit reached");
        }

        Ok(())
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

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }
}
