use serde::{Deserialize, Serialize};

use crate::server::kind::ServerKind;

fn default_address() -> String {
    "0.0.0.0".to_string()
}

fn default_retry_delay() -> u64 {
    15000
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerSettings {
    pub port: u16,
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(flatten)]
    pub kind: ServerKind,
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_settings_default_address() {
        let toml_str = r#"
            port = 7171
            type = "tcp"
        "#;
        let settings: ServerSettings =
            toml::from_str(toml_str).expect("failed to deserialize TCP settings from TOML");
        assert_eq!(settings.address, "0.0.0.0");
        assert_eq!(settings.port, 7171);
        assert_eq!(settings.retry_delay_ms, 15000);
        assert!(matches!(settings.kind, ServerKind::Tcp { .. }));
    }

    #[test]
    fn server_settings_deserialize_http() {
        let toml_str = r#"
            port = 8080
            type = "http"
        "#;
        let settings: ServerSettings =
            toml::from_str(toml_str).expect("failed to deserialize HTTP settings from TOML");
        assert_eq!(settings.port, 8080);
        assert!(matches!(settings.kind, ServerKind::Http { .. }));
    }

    #[test]
    fn server_settings_custom_fields() {
        let toml_str = r#"
            port = 7171
            type = "tcp"
            channel_capacity = 512
            max_buffer_size = 8192
            max_connections = 50
        "#;
        let settings: ServerSettings =
            toml::from_str(toml_str).expect("failed to deserialize custom TCP settings from TOML");
        if let ServerKind::Tcp {
            channel_capacity,
            max_buffer_size,
            max_connections,
            ..
        } = &settings.kind
        {
            assert_eq!(*channel_capacity, 512);
            assert_eq!(*max_buffer_size, 8192);
            assert_eq!(*max_connections, 50);
        } else {
            panic!("expected Tcp variant");
        }
    }
}
