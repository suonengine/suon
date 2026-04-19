use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Configuration for the Suon root plugin bootstrap.
#[derive(Resource, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Number of worker threads used by Bevy task pools.
    pub threads: usize,

    /// Seconds between each app loop when `schedule_runner` is enabled.
    pub event_loop: f64,

    /// Seconds for Bevy's fixed timestep resource.
    pub fixed_event_loop: f64,

    /// Whether Suon should install `ScheduleRunnerPlugin`.
    pub schedule_runner: bool,
}

impl Settings {
    /// Path to the root Suon settings file.
    pub const PATH: &'static str = "Settings.toml";

    /// Loads the settings file or creates it with defaults when it does not exist.
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
        let config = fs::read_to_string(path).context("Failed to read Suon settings file")?;
        toml::from_str(&config).context("Failed to parse Suon settings as TOML")
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        let default_config = Self::default();
        let config = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default Suon settings")?;
        {
            let mut file = File::create(path).context("Failed to create the Suon settings file")?;
            file.write_all(config.as_bytes())
                .context("Failed to write the default Suon settings file")?;
            file.sync_all()
                .context("Failed to flush the default Suon settings file")?;
        }
        Self::load_at(path)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            threads: 2,
            event_loop: 1.0 / 60.0,
            fixed_event_loop: 1.0 / 20.0,
            schedule_runner: true,
        }
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
        let settings = Settings::default();
        let serialized =
            toml::to_string(&settings).expect("Default settings should serialize to TOML");
        let deserialized: Settings =
            toml::from_str(&serialized).expect("Serialized settings should parse back");

        assert_eq!(
            deserialized, settings,
            "Serialized settings should preserve the Suon bootstrap configuration"
        );
    }

    #[test]
    fn load_or_default_should_create_the_configuration_file_when_it_is_missing() {
        let path = unique_temp_path("suon-settings-create");
        if path.exists() {
            fs::remove_file(&path).expect("The temp settings file should be removed");
        }

        let settings = Settings::load_or_default_at(&path)
            .expect("load_or_default_at should create default settings");

        assert!(
            path.exists(),
            "load_or_default_at should create the settings file when it does not exist"
        );

        assert_eq!(
            settings,
            Settings::default(),
            "The created configuration should match the default Suon settings"
        );

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }

    #[test]
    fn load_or_default_should_load_an_existing_configuration_file() {
        let path = unique_temp_path("suon-settings-load");

        let expected = Settings {
            threads: 8,
            event_loop: 0.25,
            fixed_event_loop: 0.5,
            schedule_runner: false,
        };

        fs::write(
            &path,
            toml::to_string_pretty(&expected)
                .expect("The expected settings should serialize to TOML"),
        )
        .expect("The test should write a custom settings file");

        let loaded = Settings::load_or_default_at(&path)
            .expect("load_or_default_at should load the existing file");

        assert_eq!(
            loaded, expected,
            "load_or_default_at should preserve the configured Suon settings"
        );

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }
}
