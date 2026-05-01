use anyhow::Context;
use bevy::prelude::*;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use suon_serde::{DocumentedToml, prelude::*};

use crate::seed::splitmix64_value;

/// Settings used to seed Suon's deterministic RNG resources.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RngSettings {
    /// Server seed used for deterministic gameplay randomness.
    /// The default file is created with the current Unix timestamp in milliseconds.
    seed: u64,
}

impl RngSettings {
    /// Path to the RNG settings file.
    pub const PATH: &'static str = "settings/RngSettings.toml";

    /// Creates a settings snapshot from a specific seed.
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Loads the settings file or creates it with a timestamp seed when it does not exist.
    pub fn load_or_default() -> anyhow::Result<Self> {
        Self::load_or_default_at(Path::new(Self::PATH))
    }

    /// Returns the configured server seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the seed used by [`FastRandom`](crate::FastRandom).
    pub(crate) fn fast_seed(&self) -> u64 {
        splitmix64_value(self.seed)
    }

    /// Returns a log-safe summary of the RNG settings.
    pub(crate) fn summary(&self) -> String {
        format!("seed={}, fast_seed={}", self.seed, self.fast_seed())
    }

    fn load_or_default_at(path: &Path) -> anyhow::Result<Self> {
        if path.exists() {
            info!("Loading settings from '{}'", path.display());
            return Self::load_at(path);
        }

        warn!(
            "Settings file '{}' not found; creating defaults.",
            path.display()
        );
        Self::create_at(path)
    }

    fn load_at(path: &Path) -> anyhow::Result<Self> {
        debug!("Reading settings from '{}'", path.display());

        let config = fs::read_to_string(path).context("Failed to read RNG settings file")?;
        let settings = toml::from_str(&config).context("Failed to parse RNG settings as TOML")?;

        info!("Loaded settings from '{}'", path.display());
        trace!("Loaded RNG settings: {:?}", settings);

        Ok(settings)
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        let default_config = Self::default();
        Self::write_at(path, &default_config)?;
        Self::load_at(path)
    }

    fn write_at(path: &Path, settings: &Self) -> anyhow::Result<()> {
        let config =
            write_documented_toml(settings).context("Failed to serialize default RNG settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        let mut file = File::create(path).context("Failed to create the RNG settings file")?;
        file.write_all(config.as_bytes())
            .context("Failed to write the RNG settings file")?;

        file.sync_all()
            .context("Failed to flush the RNG settings file")?;

        Ok(())
    }
}

impl Default for RngSettings {
    fn default() -> Self {
        Self {
            seed: current_time_seed(),
        }
    }
}

fn current_time_seed() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after the unix epoch");

    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{CurrentDirGuard, cwd_lock, unique_temp_dir};
    use std::fs;

    #[test]
    fn new_should_store_the_configured_seed() {
        let settings = RngSettings::new(123);

        assert_eq!(settings.seed(), 123);
    }

    #[test]
    fn settings_roundtrip_through_toml() {
        let settings = RngSettings::new(123);
        let serialized = toml::to_string(&settings).expect("RNG settings should serialize to TOML");
        let deserialized: RngSettings =
            toml::from_str(&serialized).expect("Serialized settings should parse back");

        assert_eq!(deserialized, settings);
    }

    #[test]
    fn fast_seed_should_be_derived_deterministically_from_server_seed() {
        let settings = RngSettings::new(123);

        assert_eq!(settings.fast_seed(), RngSettings::new(123).fast_seed());
        assert_ne!(settings.fast_seed(), settings.seed());
    }

    #[test]
    fn summary_should_include_server_and_fast_seed() {
        let settings = RngSettings::new(123);
        let summary = settings.summary();

        assert!(summary.contains("seed=123"));
        assert!(summary.contains(&format!("fast_seed={}", settings.fast_seed())));
    }

    #[test]
    fn load_or_default_should_create_settings_with_timestamp_seed() {
        let temp_dir = unique_temp_dir("suon-rng-settings");
        let path = temp_dir.join("RngSettings.toml");

        let before = current_time_seed();
        let settings = RngSettings::load_or_default_at(&path)
            .expect("load_or_default_at should create RNG settings");
        let after = current_time_seed();

        assert!(path.exists());
        assert!((before..=after).contains(&settings.seed()));

        let written = fs::read_to_string(&path).expect("the RNG settings file should be readable");
        assert!(written.contains("# Settings used to seed Suon's deterministic RNG resources."));
        assert!(written.contains("seed = "));

        fs::remove_dir_all(temp_dir).expect("the temp RNG settings directory should be removed");
    }

    #[test]
    fn load_or_default_should_load_existing_settings() {
        let temp_dir = unique_temp_dir("suon-rng-settings");
        let path = temp_dir.join("RngSettings.toml");
        fs::create_dir_all(&temp_dir).expect("the temp RNG settings directory should be created");
        fs::write(&path, "seed = 987\n").expect("the test settings file should be written");

        let settings = RngSettings::load_or_default_at(&path)
            .expect("load_or_default_at should load existing RNG settings");

        assert_eq!(settings, RngSettings::new(987));

        fs::remove_dir_all(temp_dir).expect("the temp RNG settings directory should be removed");
    }

    #[test]
    fn load_or_default_should_use_the_default_settings_path() {
        let _lock = cwd_lock()
            .lock()
            .expect("the RNG settings test should acquire the cwd lock");

        let temp_dir = unique_temp_dir("suon-rng-settings-cwd");
        fs::create_dir_all(&temp_dir).expect("the temp RNG settings directory should be created");

        {
            let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

            let settings = RngSettings::load_or_default()
                .expect("load_or_default should create settings at the default path");

            assert!(temp_dir.join(RngSettings::PATH).exists());
            assert!(settings.seed() > 0);
        }

        fs::remove_dir_all(temp_dir).expect("the temp RNG settings directory should be removed");
    }
}
