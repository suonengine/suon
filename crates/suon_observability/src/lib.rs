//! Observability bootstrap for Suon applications.
//!
//! This crate owns the TOML-backed settings and Bevy plugin wiring for logging
//! and diagnostics used by headless Suon apps.

use anyhow::Context;
use bevy::{
    diagnostic::{
        DiagnosticsPlugin, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    log::LogPlugin,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use suon_serde::{DocumentedToml, prelude::*};

pub mod prelude {
    pub use crate::{ObservabilityPlugin, ObservabilitySettings};
}

/// Controls logs and runtime diagnostics exposed by the server.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug, PartialEq)]
pub struct ObservabilitySettings {
    /// Enables normal application logs.
    /// Turn this off only if logging is configured elsewhere.
    pub log: bool,

    /// Enables the diagnostics infrastructure used by the options below.
    /// This alone does not print or expose much until another diagnostic is enabled.
    pub metrics: bool,

    /// Periodically writes diagnostic counters to the log output.
    /// Useful for lightweight visibility in local development and servers without dashboards.
    pub log_metrics: bool,

    /// Collects frame and tick timing statistics.
    /// Useful when tuning server cadence or investigating stalls.
    pub frame_time: bool,

    /// Tracks how many ECS entities currently exist.
    /// Helpful for spotting leaks or unexpected world growth over time.
    pub entity_count: bool,

    /// Collects host machine information such as CPU and memory details.
    /// Usually most useful in development, debugging, or operator environments.
    pub system_information: bool,
}

impl ObservabilitySettings {
    /// Path to the observability settings file.
    pub const PATH: &'static str = "settings/ObservabilitySettings.toml";

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

        let config =
            fs::read_to_string(path).context("Failed to read observability settings file")?;

        info!("Successfully read configuration file '{}'", path.display());

        let settings =
            toml::from_str(&config).context("Failed to parse observability settings as TOML")?;

        trace!("Loaded settings: {:?}", settings);

        Ok(settings)
    }

    fn create_at(path: &Path) -> anyhow::Result<Self> {
        info!("Creating default configuration file '{}'", path.display());
        let default_config = Self::default();

        Self::write_at(path, &default_config)?;

        info!(
            "Default configuration written to '{}'. Reloading from file.",
            path.display()
        );

        Self::load_at(path)
    }

    fn write_at(path: &Path, default_config: &Self) -> anyhow::Result<()> {
        debug!("Rendering documented configuration");

        let config = write_documented_toml(default_config)
            .context("Failed to serialize default observability settings")?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        debug!("Creating configuration file at '{}'", path.display());

        let mut file =
            File::create(path).context("Failed to create the observability settings file")?;

        debug!("Writing default configuration to file");

        file.write_all(config.as_bytes())
            .context("Failed to write the default observability settings file")?;

        file.sync_all()
            .context("Failed to flush the default observability settings file")?;

        Ok(())
    }

    /// Returns a log-safe summary of observability features.
    pub fn summary(self) -> String {
        format!(
            "log={}, metrics={}, log_metrics={}, frame_time={}, entity_count={}, \
             system_information={}",
            self.log,
            self.metrics,
            self.log_metrics,
            self.frame_time,
            self.entity_count,
            self.system_information
        )
    }
}

impl Default for ObservabilitySettings {
    fn default() -> Self {
        Self {
            log: true,
            metrics: false,
            log_metrics: false,
            frame_time: false,
            entity_count: false,
            system_information: false,
        }
    }
}

/// Plugin that loads `settings/ObservabilitySettings.toml` and installs Bevy logging and
/// diagnostics plugins accordingly.
pub struct ObservabilityPlugin;

impl Plugin for ObservabilityPlugin {
    fn build(&self, app: &mut App) {
        let settings = ObservabilitySettings::load_or_default()
            .expect("Failed to load observability settings.");
        let settings_summary = settings.summary();

        let metrics_enabled = settings.metrics
            || settings.log_metrics
            || settings.frame_time
            || settings.entity_count
            || settings.system_information;

        app.insert_resource(settings);

        if metrics_enabled {
            app.add_plugins(DiagnosticsPlugin);
        }

        if settings.log {
            app.add_plugins(LogPlugin::default());
        }

        if settings.log_metrics {
            app.add_plugins(LogDiagnosticsPlugin::default());
        }

        if settings.frame_time {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        if settings.entity_count {
            app.add_plugins(EntityCountDiagnosticsPlugin::default());
        }

        if settings.system_information {
            app.add_plugins(SystemInformationDiagnosticsPlugin);
        }

        info!(
            "Starting the observability systems with {}; diagnostics_enabled={}",
            settings_summary, metrics_enabled
        );
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
        let settings = ObservabilitySettings::default();

        let serialized = toml::to_string(&settings)
            .expect("Default observability settings should serialize to TOML");

        let deserialized: ObservabilitySettings =
            toml::from_str(&serialized).expect("Serialized settings should parse back");

        assert_eq!(
            deserialized, settings,
            "Serialized settings should preserve the observability configuration"
        );
    }

    #[test]
    fn load_or_default_should_create_the_configuration_file_when_it_is_missing() {
        let path = unique_temp_path("suon-observability-settings-create");
        if path.exists() {
            fs::remove_file(&path).expect("The temp settings file should be removed");
        }

        let settings = ObservabilitySettings::load_or_default_at(&path)
            .expect("load_or_default_at should create default settings");

        assert!(
            path.exists(),
            "load_or_default_at should create the settings file when it does not exist"
        );

        assert_eq!(
            settings,
            ObservabilitySettings::default(),
            "The created configuration should match the default observability settings"
        );

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }

    #[test]
    fn load_or_default_should_load_an_existing_configuration_file() {
        let path = unique_temp_path("suon-observability-settings-load");

        let expected = ObservabilitySettings {
            log: false,
            metrics: true,
            log_metrics: false,
            frame_time: false,
            entity_count: true,
            system_information: false,
        };

        fs::write(
            &path,
            toml::to_string_pretty(&expected)
                .expect("The expected settings should serialize to TOML"),
        )
        .expect("The test should write a custom settings file");

        let loaded = ObservabilitySettings::load_or_default_at(&path)
            .expect("load_or_default_at should load the existing file");

        assert_eq!(
            loaded, expected,
            "load_or_default_at should preserve the configured observability settings"
        );

        fs::remove_file(&path).expect("The temp settings file should be removed");
    }
}
