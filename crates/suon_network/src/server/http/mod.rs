pub(crate) mod acceptor;
pub(crate) mod manager;
pub(crate) mod request;
pub(crate) mod session;
pub(crate) mod task;

use serde::{Deserialize, Serialize};

fn default_max_connections() -> u32 {
    100
}

fn default_rate_burst() -> u32 {
    50
}

fn default_max_headers() -> usize {
    32
}

/// Configuration for an HTTP listener port.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct HttpSettings {
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_rate_burst")]
    pub rate_burst: u32,
    #[serde(default = "default_max_headers")]
    pub max_headers: usize,
}

impl Default for HttpSettings {
    fn default() -> Self {
        HttpSettings {
            max_connections: default_max_connections(),
            rate_burst: default_rate_burst(),
            max_headers: default_max_headers(),
        }
    }
}
