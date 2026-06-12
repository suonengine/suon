use std::path::Path;
use tracing::{error, info};

use crate::{
    server::{kind::ServerKind, settings::ServerSettings, tcp::ProtocolSettings},
    settings_error::SettingsError,
};

const FILE: &str = "NetworkSettings.toml";

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct NetworkSettings {
    pub worker_threads: usize,
    pub server: Vec<ServerSettings>,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        NetworkSettings {
            worker_threads: 2,
            server: vec![
                ServerSettings {
                    port: 7171,
                    address: "0.0.0.0".into(),
                    kind: ServerKind::Tcp {
                        protocol: ProtocolSettings {
                            header_size: 6,
                            has_checksum: true,
                            uses_xtea: true,
                            uses_rsa: true,
                        },
                        flush_interval_ms: 10,
                        encryption: Default::default(),
                        channel_capacity: 1024,
                        max_buffer_size: 4096,
                        max_connections: 100,
                    },
                    retry_delay_ms: 15000,
                },
                ServerSettings {
                    port: 7172,
                    address: "0.0.0.0".into(),
                    kind: ServerKind::Tcp {
                        protocol: ProtocolSettings {
                            header_size: 6,
                            has_checksum: true,
                            uses_xtea: true,
                            uses_rsa: false,
                        },
                        flush_interval_ms: 10,
                        encryption: Default::default(),
                        channel_capacity: 1024,
                        max_buffer_size: 4096,
                        max_connections: 100,
                    },
                    retry_delay_ms: 15000,
                },
                ServerSettings {
                    port: 8080,
                    address: "0.0.0.0".into(),
                    kind: ServerKind::Http {
                        max_connections: 100,
                        rate_burst: 50,
                        max_headers: 32,
                    },
                    retry_delay_ms: 15000,
                },
            ],
        }
    }
}

impl NetworkSettings {
    fn read(path: &Path) -> Result<Self, SettingsError> {
        let content = std::fs::read_to_string(path)?;
        let settings: NetworkSettings = toml::from_str(&content)?;

        for server_settings in &settings.server {
            if server_settings.port == 0 {
                return Err(SettingsError::Validation(
                    "server port must not be 0".into(),
                ));
            }
        }

        let mut ports = std::collections::HashSet::new();
        for server_settings in &settings.server {
            if !ports.insert((server_settings.port, server_settings.kind.clone())) {
                return Err(SettingsError::Validation(format!(
                    "duplicate {} server on port {}",
                    server_settings.kind.as_str(),
                    server_settings.port
                )));
            }
        }

        Ok(settings)
    }

    fn write(&self, path: &Path) -> Result<(), SettingsError> {
        let content = toml::to_string(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn load() -> Self {
        let path = Path::new(FILE);
        info!(target: "Settings", "Loading network settings from {FILE}");

        match Self::read(path) {
            Ok(settings) => settings,
            Err(err) => {
                let not_found = matches!(
                    &err,
                    SettingsError::Io(io_error)
                        if io_error.kind() == std::io::ErrorKind::NotFound
                );

                if not_found {
                    let settings = NetworkSettings::default();
                    settings.write(path).unwrap_or_else(|write_err| {
                        error!(target: "Settings", "Failed to write default settings: {write_err}");
                        panic!("Failed to write default settings: {write_err}")
                    });
                    settings
                } else {
                    error!(target: "Settings", "Failed to load settings from {FILE}: {err}");
                    panic!("Failed to load settings from {FILE}: {err}");
                }
            }
        }
    }
}

impl std::fmt::Display for NetworkSettings {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let servers: Vec<String> = self
            .server
            .iter()
            .map(|server_settings| {
                format!(
                    "{}:{}/{}",
                    server_settings.address,
                    server_settings.port,
                    server_settings.kind.as_str()
                )
            })
            .collect();

        write!(
            formatter,
            "workers={} server=[{}]",
            self.worker_threads,
            servers.join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_settings_default() {
        let settings = NetworkSettings::default();
        assert_eq!(settings.worker_threads, 2);
        assert_eq!(settings.server.len(), 3);
    }

    #[test]
    fn network_settings_read_write_roundtrip() {
        let settings = NetworkSettings::default();
        let dir = std::env::temp_dir().join("suon_test_settings");
        let path = dir.join("NetworkSettings.toml");
        if path.exists() {
            std::fs::remove_file(&path)
                .expect("failed to remove leftover settings file before test");
        }

        settings
            .write(&path)
            .expect("failed to write default settings to temp file");
        let loaded = NetworkSettings::read(&path).expect("failed to read settings from temp file");
        assert_eq!(loaded.worker_threads, settings.worker_threads);
        assert_eq!(loaded.server.len(), settings.server.len());
        std::fs::remove_file(&path).expect("failed to remove settings file after test");
    }

    #[test]
    fn network_settings_read_file_not_found() {
        let path = std::env::temp_dir().join("suon_test_settings_does_not_exist.toml");
        let result = NetworkSettings::read(&path);
        assert!(matches!(result, Err(SettingsError::Io(_))));
    }

    #[test]
    fn network_settings_read_invalid_toml() {
        let dir = std::env::temp_dir().join("suon_test_settings_invalid");
        let path = dir.join("NetworkSettings.toml");
        std::fs::create_dir_all(&dir).expect("failed to create temp directory for test");
        std::fs::write(&path, b"invalid toml {{{")
            .expect("failed to write invalid toml to temp file");

        let result = NetworkSettings::read(&path);
        assert!(matches!(result, Err(SettingsError::Parse(_))));

        std::fs::remove_file(&path).expect("failed to remove settings file after test");
    }

    #[test]
    fn network_settings_display_contains_servers() {
        let settings = NetworkSettings::default();
        let display = settings.to_string();
        assert!(display.contains("7171"));
        assert!(display.contains("7172"));
        assert!(display.contains("8080"));
        assert!(display.contains("workers="));
    }
}
