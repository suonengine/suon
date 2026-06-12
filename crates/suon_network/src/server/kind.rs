use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::server::tcp::{EncryptionSettings, ProtocolSettings};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerKind {
    Tcp {
        #[serde(default)]
        protocol: ProtocolSettings,
        #[serde(rename = "flush_interval_ms", with = "suon_serde::duration_ms")]
        flush_interval: Duration,
        #[serde(default)]
        encryption: EncryptionSettings,
        channel_capacity: usize,
        max_buffer_size: usize,
        max_connections: u32,
    },
    Http {
        max_connections: u32,
        rate_burst: u32,
        max_headers: usize,
    },
}

impl Default for ServerKind {
    fn default() -> Self {
        ServerKind::Tcp {
            protocol: ProtocolSettings::default(),
            flush_interval: Duration::from_millis(10),
            encryption: EncryptionSettings::default(),
            channel_capacity: 1024,
            max_buffer_size: 4096,
            max_connections: 100,
        }
    }
}

impl ServerKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerKind::Tcp { .. } => "tcp",
            ServerKind::Http { .. } => "http",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_kind_default_is_tcp() {
        let kind = ServerKind::default();
        assert!(matches!(kind, ServerKind::Tcp { .. }));
        assert_eq!(kind.as_str(), "tcp");
    }

    #[test]
    fn server_kind_http_as_str() {
        let kind = ServerKind::Http {
            max_connections: 100,
            rate_burst: 50,
            max_headers: 32,
        };
        assert_eq!(kind.as_str(), "http");
    }

    #[test]
    fn server_kind_default_values() {
        let kind = ServerKind::default();
        if let ServerKind::Tcp {
            channel_capacity,
            max_buffer_size,
            max_connections,
            flush_interval,
            ..
        } = kind
        {
            assert_eq!(channel_capacity, 1024);
            assert_eq!(max_buffer_size, 4096);
            assert_eq!(max_connections, 100);
            assert_eq!(flush_interval, Duration::from_millis(10));
        } else {
            panic!("expected Tcp variant");
        }
    }
}
