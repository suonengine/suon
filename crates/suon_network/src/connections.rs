use std::sync::Arc;

use suon_macros::Resource;

use crate::connection::{handle::ConnectionHandle, id::ConnectionId, manager::ConnectionManager};

/// Global registry of all active connections.
///
/// Wraps a single [`ConnectionManager`] that assigns globally-unique
/// connection identifiers.  Inserted into [`Resources`] so Lua bindings
/// and task handlers can look up and interact with any active connection.
#[derive(Clone, Resource)]
pub struct Connections {
    pub manager: Arc<ConnectionManager>,
}

impl Connections {
    pub fn new() -> Self {
        Connections {
            manager: Arc::new(ConnectionManager::new(0)),
        }
    }

    /// Looks up a [`ConnectionHandle`] by its [`ConnectionId`].
    pub fn get(&self, identifier: ConnectionId) -> Option<ConnectionHandle> {
        self.manager.get(identifier)
    }

    /// Send raw bytes to the identified connection.
    pub fn send(&self, id: u64, data: Vec<u8>) -> Result<(), String> {
        let identifier = ConnectionId::from_u64(id);
        let handle = self
            .manager
            .get(identifier)
            .ok_or_else(|| format!("connection {id} not found"))?;

        handle
            .send(data)
            .map_err(|error| format!("send failed: {error}"))
    }

    /// Send raw bytes, bypassing protocol framing/encryption.
    pub fn send_raw(&self, id: u64, data: Vec<u8>) -> Result<(), String> {
        let identifier = ConnectionId::from_u64(id);
        let handle = self
            .manager
            .get(identifier)
            .ok_or_else(|| format!("connection {id} not found"))?;

        handle
            .send_raw(data)
            .map_err(|error| format!("send_raw failed: {error}"))
    }

    /// Gracefully close the connection.
    pub fn close(&self, id: u64) -> Result<(), String> {
        let id = ConnectionId::from_u64(id);
        let handle = self
            .manager
            .get(id)
            .ok_or_else(|| format!("connection {id} not found"))?;

        handle
            .close()
            .map_err(|error| format!("close failed: {error}"))
    }
}

impl Default for Connections {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let connections = Connections::new();
        let identifier = ConnectionId::new(0, 1);
        assert!(connections.get(identifier).is_none());
    }

    #[test]
    fn register_and_get() {
        use crate::server::tcp::ProtocolSettings;
        use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

        let connections = Connections::new();
        let (sender, _receiver) = crossbeam_channel::bounded(16);
        let settings = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        };
        let peer = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7000));
        let identifier = connections.manager.register(peer, settings, sender);
        assert!(connections.get(identifier).is_some());
    }

    #[test]
    fn send_missing_connection_returns_error() {
        let connections = Connections::new();
        let result = connections.send(999, vec![1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn close_missing_connection_returns_error() {
        let connections = Connections::new();
        let result = connections.close(999);
        assert!(result.is_err());
    }
}
