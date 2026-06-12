use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::server::{
    kind::ServerKind,
    settings::ServerSettings,
    tcp::{EncryptionSettings, ProtocolSettings},
};

/// Configuration for a TCP listener port.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct TcpSettings {
    #[serde(default)]
    pub protocol: ProtocolSettings,
    #[serde(rename = "flush_interval_ms", with = "suon_serde::duration_ms")]
    pub flush_interval: Duration,
    #[serde(default)]
    pub encryption: EncryptionSettings,
    pub channel_capacity: usize,
    pub max_buffer_size: usize,
    pub max_connections: u32,
}

impl Default for TcpSettings {
    fn default() -> Self {
        TcpSettings {
            protocol: ProtocolSettings::default(),
            flush_interval: Duration::from_millis(10),
            encryption: EncryptionSettings::default(),
            channel_capacity: 1024,
            max_buffer_size: 4096,
            max_connections: 100,
        }
    }
}

impl TcpSettings {
    pub fn from_settings(settings: &ServerSettings) -> Self {
        match &settings.kind {
            ServerKind::Tcp {
                protocol,
                flush_interval,
                encryption,
                channel_capacity,
                max_buffer_size,
                max_connections,
            } => TcpSettings {
                protocol: *protocol,
                flush_interval: *flush_interval,
                encryption: *encryption,
                channel_capacity: *channel_capacity,
                max_buffer_size: *max_buffer_size,
                max_connections: *max_connections,
            },
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{kind::ServerKind, settings::ServerSettings};
    use std::time::Duration;

    fn make_settings() -> ServerSettings {
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
                flush_interval: Duration::from_millis(10),
                encryption: EncryptionSettings {
                    incoming: true,
                    outgoing: false,
                },
                channel_capacity: 512,
                max_buffer_size: 8192,
                max_connections: 50,
            },
            retry_delay: Duration::from_millis(5000),
        }
    }

    #[test]
    fn tcp_settings_from_tcp_server() {
        let settings = make_settings();
        let tcp = TcpSettings::from_settings(&settings);
        assert_eq!(tcp.protocol.header_size, 6);
        assert!(tcp.protocol.has_checksum);
        assert!(tcp.protocol.uses_xtea);
        assert!(tcp.protocol.uses_rsa);
        assert_eq!(tcp.flush_interval, Duration::from_millis(10));
        assert!(tcp.encryption.incoming);
        assert!(!tcp.encryption.outgoing);
        assert_eq!(tcp.channel_capacity, 512);
        assert_eq!(tcp.max_buffer_size, 8192);
        assert_eq!(tcp.max_connections, 50);
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn tcp_settings_from_http_panics() {
        let settings = ServerSettings {
            port: 8080,
            address: "0.0.0.0".into(),
            kind: ServerKind::Http {
                max_connections: 100,
                rate_burst: 50,
                max_headers: 32,
            },
            retry_delay: Duration::from_millis(15000),
        };
        TcpSettings::from_settings(&settings);
    }
}
