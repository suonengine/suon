pub(crate) mod acceptor;
pub(crate) mod manager;
pub(crate) mod request;
pub(crate) mod session;
pub(crate) mod task;

use serde::{Deserialize, Serialize};

/// Configuration for an HTTP listener port.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct HttpSettings {
    pub max_connections: u32,
    pub rate_burst: u32,
    pub max_headers: usize,
}

impl Default for HttpSettings {
    fn default() -> Self {
        HttpSettings {
            max_connections: 100,
            rate_burst: 50,
            max_headers: 32,
        }
    }
}
