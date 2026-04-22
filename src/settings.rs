use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use suon_serde::{DocumentedToml, prelude::*};

/// Core runtime settings for a Suon server process.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Number of worker threads available for background work.
    /// Increase this when your server has more CPU headroom.
    pub threads: usize,

    /// Main app loop frequency, in hertz.
    /// Higher values make the app tick more often and consume more CPU.
    pub event_loop_hz: f64,

    /// Fixed update frequency, in hertz, for deterministic gameplay systems.
    /// Typical uses include movement, combat, and other time-sensitive logic.
    pub fixed_event_loop_hz: f64,

    /// Enables Suon's built-in app loop runner.
    /// Disable this if your host application drives the Bevy schedule itself.
    pub schedule_runner: bool,
}

impl Settings {
    /// Path to the root Suon settings file.
    pub const PATH: &'static str = "settings/Settings.toml";

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
        debug!("Attempting to read configuration from '{}'", path.display());

        let config = fs::read_to_string(path).context("Failed to read Suon settings file")?;

        info!("Successfully read configuration file '{}'", path.display());

        let settings = toml::from_str(&config).context("Failed to parse Suon settings as TOML")?;

        trace!("Loaded settings: {:?}", settings);

        Ok(settings)
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        info!("Creating default configuration file '{}'", path.display());

        let default_config = Self::default();

        debug!("Rendering documented default configuration");

        let config = write_documented_toml(&default_config)
            .context("Failed to serialize default Suon settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        debug!("Creating configuration file at '{}'", path.display());

        let mut file = File::create(path).context("Failed to create the Suon settings file")?;

        debug!("Writing default configuration to file");

        file.write_all(config.as_bytes())
            .context("Failed to write the default Suon settings file")?;

        file.sync_all()
            .context("Failed to flush the default Suon settings file")?;

        info!(
            "Default configuration written to '{}'. Reloading from file.",
            path.display()
        );

        Self::load_at(path)
    }

    /// Returns a log-safe summary of the bootstrap settings.
    pub fn summary(self) -> String {
        format!(
            "threads={}, schedule_runner={}, event_loop_hz={:.2}, fixed_event_loop_hz={:.2}",
            self.threads, self.schedule_runner, self.event_loop_hz, self.fixed_event_loop_hz
        )
    }

    /// Returns the configured app loop frequency in hertz.
    pub fn event_loop_hz(self) -> f64 {
        self.event_loop_hz
    }

    /// Returns the configured fixed timestep frequency in hertz.
    pub fn fixed_event_loop_hz(self) -> f64 {
        self.fixed_event_loop_hz
    }

    /// Returns the app loop interval in seconds.
    pub fn event_loop_seconds(self) -> f64 {
        1.0 / self.event_loop_hz
    }

    /// Returns the fixed timestep interval in seconds.
    pub fn fixed_event_loop_seconds(self) -> f64 {
        1.0 / self.fixed_event_loop_hz
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            threads: 2,
            event_loop_hz: 60.0,
            fixed_event_loop_hz: 20.0,
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

        let written = fs::read_to_string(&path).expect("the settings file should be readable");
        assert!(written.contains("# Configuration for the Suon root plugin bootstrap."));
        assert!(written.contains("event_loop_hz = 60.0"));

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }

    #[test]
    fn load_or_default_should_load_an_existing_configuration_file() {
        let path = unique_temp_path("suon-settings-load");

        let expected = Settings {
            threads: 8,
            event_loop_hz: 4.0,
            fixed_event_loop_hz: 2.0,
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
