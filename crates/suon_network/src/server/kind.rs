use serde::{Deserialize, Serialize};

use crate::server::tcp::{EncryptionSettings, ProtocolSettings};

fn default_flush_interval() -> u64 {
    10
}

fn default_channel_capacity() -> usize {
    1024
}

fn default_max_buffer_size() -> usize {
    4096
}

fn default_max_connections() -> u32 {
    100
}

fn default_rate_burst() -> u32 {
    50
}

fn default_max_headers() -> usize {
    32
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerKind {
    Tcp {
        #[serde(default)]
        protocol: ProtocolSettings,
        #[serde(default = "default_flush_interval")]
        flush_interval_ms: u64,
        #[serde(default)]
        encryption: EncryptionSettings,
        #[serde(default = "default_channel_capacity")]
        channel_capacity: usize,
        #[serde(default = "default_max_buffer_size")]
        max_buffer_size: usize,
        #[serde(default = "default_max_connections")]
        max_connections: u32,
    },
    Http {
        #[serde(default = "default_max_connections")]
        max_connections: u32,
        #[serde(default = "default_rate_burst")]
        rate_burst: u32,
        #[serde(default = "default_max_headers")]
        max_headers: usize,
    },
}

impl Default for ServerKind {
    fn default() -> Self {
        ServerKind::Tcp {
            protocol: ProtocolSettings::default(),
            flush_interval_ms: default_flush_interval(),
            encryption: EncryptionSettings::default(),
            channel_capacity: default_channel_capacity(),
            max_buffer_size: default_max_buffer_size(),
            max_connections: default_max_connections(),
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
            flush_interval_ms,
            ..
        } = kind
        {
            assert_eq!(channel_capacity, 1024);
            assert_eq!(max_buffer_size, 4096);
            assert_eq!(max_connections, 100);
            assert_eq!(flush_interval_ms, 10);
        } else {
            panic!("expected Tcp variant");
        }
    }
}
