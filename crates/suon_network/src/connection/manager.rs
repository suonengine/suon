use std::{
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use tracing::{trace, warn};

use dashmap::DashMap;

use crate::{
    connection::{
        handle::ConnectionHandle, id::ConnectionId, info::ConnectionInfo, stats::ConnectionStats,
    },
    protocol::command::CommandSender,
    server::tcp::ProtocolSettings,
};

/// ID namespace for a listener port.
pub(crate) type PortNamespace = u32;

/// Centralized registry of all active connections.
///
/// Uses lock-striping via [`DashMap`] so that concurrent registrations
/// and removals do not contend on a single [`Mutex`].
pub struct ConnectionManager {
    next_id: AtomicU64,
    connections: DashMap<u64, (ConnectionHandle, ProtocolSettings, Instant)>,
    port_namespace: PortNamespace,
    pub stats: Arc<ConnectionStats>,
}

impl ConnectionManager {
    /// Creates a new manager for the given listener port namespace.
    ///
    /// Every connection registered through this manager will bear a
    /// [`ConnectionId`] whose upper 32 bits equal `port_namespace`,
    /// preventing ID collisions across multiple listener ports.
    pub fn new(port_namespace: PortNamespace) -> Self {
        ConnectionManager {
            next_id: AtomicU64::new(1),
            connections: DashMap::new(),
            port_namespace,
            stats: Arc::new(ConnectionStats::default()),
        }
    }

    /// Registers a new connection and returns its assigned ID and handle.
    pub fn register(
        &self,
        peer: SocketAddr,
        protocol: ProtocolSettings,
        sender: CommandSender,
    ) -> ConnectionId {
        let seq = self.next_id.fetch_add(1, Ordering::Relaxed) as u32;
        let id = ConnectionId::new(self.port_namespace, seq);
        let handle = ConnectionHandle::new(id, peer, sender);
        self.connections
            .insert(id.as_u64(), (handle, protocol, Instant::now()));
        self.stats.record_accepted();
        trace!(target: "Connection", "Registered connection {id} from {peer}");
        id
    }

    /// Removes a connection from the registry.
    pub fn unregister(&self, id: ConnectionId) {
        self.connections.remove(&id.as_u64());
        self.stats.record_closed();
        trace!(target: "Connection", "Unregistered connection {id}");
    }

    /// Returns a snapshot of the connection handle, if it is still active.
    pub fn get(&self, id: ConnectionId) -> Option<ConnectionHandle> {
        self.connections
            .get(&id.as_u64())
            .map(|entry| entry.0.clone())
    }

    /// Returns the number of currently active connections.
    pub fn count(&self) -> usize {
        self.connections.len()
    }

    /// Returns a serializable list of all active connections.
    pub fn active_connections(&self) -> Vec<ConnectionInfo> {
        self.connections
            .iter()
            .map(|entry| {
                let (handle, protocol, connected_at) = entry.value();
                ConnectionInfo::new(handle.id(), handle.addr(), *protocol, *connected_at)
            })
            .collect()
    }

    /// Removes all connections and returns the count of cleaned-up entries.
    pub fn clear(&self) -> usize {
        let count = self.connections.len();
        self.connections.clear();
        warn!(target: "Connection", "Connection manager cleared {count} connections");
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn test_peer() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7000))
    }

    fn test_protocol() -> ProtocolSettings {
        ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        }
    }

    #[test]
    fn manager_new_is_empty() {
        let manager = ConnectionManager::new(0);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn manager_register_increments_count() {
        let manager = ConnectionManager::new(0);
        let (sender, _) = crossbeam_channel::bounded(16);
        let id = manager.register(test_peer(), test_protocol(), sender);
        assert_eq!(manager.count(), 1);
        assert_eq!(id.port_namespace(), 0);
        assert_eq!(id.sequence(), 1);
    }

    #[test]
    fn manager_register_multiple_increments_ids() {
        let manager = ConnectionManager::new(42);
        let (s1, _) = crossbeam_channel::bounded(16);
        let (s2, _) = crossbeam_channel::bounded(16);
        let id1 = manager.register(test_peer(), test_protocol(), s1);
        let id2 = manager.register(test_peer(), test_protocol(), s2);
        assert_eq!(id1.as_u64() + 1, id2.as_u64());
        assert_eq!(id1.port_namespace(), 42);
        assert_eq!(id2.port_namespace(), 42);
    }

    #[test]
    fn manager_register_namespace_isolation() {
        let mgr_a = ConnectionManager::new(1);
        let mgr_b = ConnectionManager::new(2);
        let (sa, _) = crossbeam_channel::bounded(16);
        let (sb, _) = crossbeam_channel::bounded(16);
        let id_a = mgr_a.register(test_peer(), test_protocol(), sa);
        let id_b = mgr_b.register(test_peer(), test_protocol(), sb);
        assert_eq!(id_a.port_namespace(), 1);
        assert_eq!(id_b.port_namespace(), 2);
        assert_ne!(id_a.as_u64(), id_b.as_u64());
    }

    #[test]
    fn manager_unregister_removes() {
        let manager = ConnectionManager::new(0);
        let (sender, _) = crossbeam_channel::bounded(16);
        let id = manager.register(test_peer(), test_protocol(), sender);
        assert_eq!(manager.count(), 1);
        manager.unregister(id);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn manager_unregister_nonexistent_is_noop() {
        let manager = ConnectionManager::new(0);
        let id = ConnectionId::new(0, 999);
        manager.unregister(id); // should not panic
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn manager_get_returns_handle() {
        let manager = ConnectionManager::new(0);
        let (sender, _) = crossbeam_channel::bounded(16);
        let id = manager.register(test_peer(), test_protocol(), sender.clone());
        let handle = manager.get(id).expect("handle should exist");
        assert_eq!(handle.id(), id);
        assert_eq!(handle.addr(), test_peer());
    }

    #[test]
    fn manager_get_nonexistent_returns_none() {
        let manager = ConnectionManager::new(0);
        let id = ConnectionId::new(0, 999);
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn manager_get_after_unregister_returns_none() {
        let manager = ConnectionManager::new(0);
        let (sender, _) = crossbeam_channel::bounded(16);
        let id = manager.register(test_peer(), test_protocol(), sender);
        manager.unregister(id);
        assert!(manager.get(id).is_none());
    }

    #[test]
    fn manager_active_connections_list() {
        let manager = ConnectionManager::new(0);
        let (s1, _) = crossbeam_channel::bounded(16);
        let (s2, _) = crossbeam_channel::bounded(16);
        let id1 = manager.register(test_peer(), test_protocol(), s1);
        manager.register(test_peer(), test_protocol(), s2);
        let list = manager.active_connections();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|c| c.id == id1));
    }

    #[test]
    fn manager_stats_tracked() {
        let manager = ConnectionManager::new(0);
        let (s1, _) = crossbeam_channel::bounded(16);
        let (s2, _) = crossbeam_channel::bounded(16);
        let id1 = manager.register(test_peer(), test_protocol(), s1);
        let id2 = manager.register(test_peer(), test_protocol(), s2);

        assert_eq!(manager.stats.total_accepted.load(Ordering::Relaxed), 2);

        manager.unregister(id1);
        manager.unregister(id2);

        assert_eq!(manager.stats.total_closed.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn manager_concurrent_registrations() {
        let manager = Arc::new(ConnectionManager::new(0));
        let mut handles = Vec::new();

        for _ in 0..10 {
            let mgr = manager.clone();
            handles.push(std::thread::spawn(move || {
                let (sender, _) = crossbeam_channel::bounded(16);
                mgr.register(test_peer(), test_protocol(), sender);
            }));
        }

        for h in handles {
            h.join().expect("test thread should join successfully");
        }

        assert_eq!(manager.count(), 10);
    }

    #[test]
    fn manager_clear_removes_all() {
        let manager = ConnectionManager::new(0);
        for _ in 0..5 {
            let (sender, _) = crossbeam_channel::bounded(16);
            manager.register(test_peer(), test_protocol(), sender);
        }
        assert_eq!(manager.count(), 5);
        let cleared = manager.clear();
        assert_eq!(cleared, 5);
        assert_eq!(manager.count(), 0);
    }
}
