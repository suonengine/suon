use std::{collections::HashMap, sync::Arc, time::Duration};

use tracing::info;

use suon_channel::Channel;
use suon_macros::Resource;
use tokio::runtime::Runtime;
use tracing::error;

use crate::{
    error::NetworkError,
    server::{binder::Binder, kind::ServerKind, settings::ServerSettings, shutdown::Shutdown},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerStatus {
    Running,
    Stopped,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub port: u16,
    pub kind: ServerKind,
    pub status: ServerStatus,
}

struct ManagedServer {
    shutdown: Shutdown,
    kind: ServerKind,
}

#[derive(Resource)]
pub struct NetworkManager {
    runtime: Arc<Runtime>,
    channel: Channel,
    servers: HashMap<u16, ManagedServer>,
}

impl NetworkManager {
    pub fn new(runtime: Arc<Runtime>, channel: Channel) -> Self {
        NetworkManager {
            runtime,
            channel,
            servers: HashMap::new(),
        }
    }

    pub fn spawn_server(&mut self, settings: ServerSettings) -> Result<(), NetworkError> {
        let port = settings.port;
        if self.servers.contains_key(&port) {
            return Err(NetworkError::AlreadyRunning(port));
        }

        let shutdown = Shutdown::new();

        self.servers.insert(
            port,
            ManagedServer {
                shutdown: shutdown.clone(),
                kind: settings.kind.clone(),
            },
        );

        let runtime = self.runtime.clone();
        let retry_delay = Duration::from_millis(settings.retry_delay_ms);

        let kind_str = settings.kind.as_str();
        info!(target: "Manager", "Spawning {kind_str} server on port {port}");

        Binder::new(
            runtime,
            self.channel.clone(),
            settings,
            shutdown,
            retry_delay,
        )
        .launch();

        Ok(())
    }

    pub fn stop(&mut self, port: u16) -> Result<(), NetworkError> {
        match self.servers.remove(&port) {
            Some(managed_server) => {
                info!(target: "Manager", "Stopping server on port {port}");
                managed_server.shutdown.trigger();
                Ok(())
            }
            None => {
                error!(target: "Manager", "Stop failed: port {port} is not running");
                Err(NetworkError::NotRunning(port))
            }
        }
    }

    pub fn restart(&mut self, settings: &ServerSettings) -> Result<(), NetworkError> {
        if let Err(e) = self.stop(settings.port) {
            error!(target: "Manager", "Stop error: {e}");
        }
        self.spawn_server(settings.clone())
    }

    pub fn status(&self) -> Vec<ServerInfo> {
        self.servers
            .iter()
            .map(|(port, managed_server)| {
                let status = ServerStatus::Running;
                ServerInfo {
                    port: *port,
                    kind: managed_server.kind.clone(),
                    status,
                }
            })
            .collect()
    }

    pub fn is_running(&self, port: u16) -> bool {
        self.servers.contains_key(&port)
    }

    pub fn shutdown_all(&mut self) {
        for (_, managed_server) in self.servers.drain() {
            managed_server.shutdown.trigger();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use suon_channel::Channel;
    use tokio::runtime::Runtime;

    use super::*;
    use crate::server::{
        kind::ServerKind,
        tcp::{EncryptionSettings, ProtocolSettings},
    };

    fn dummy_settings() -> ServerSettings {
        ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings::default(),
                flush_interval_ms: 10,
                encryption: EncryptionSettings::default(),
                channel_capacity: 16,
                max_buffer_size: 256,
                max_connections: 5,
            },
            retry_delay_ms: 100,
        }
    }

    fn make_manager() -> (NetworkManager, Arc<Runtime>, Channel) {
        let runtime = Arc::new(Runtime::new().expect("failed to build test runtime"));
        let channel = Channel::default();
        let manager = NetworkManager::new(runtime.clone(), channel.clone());
        (manager, runtime, channel)
    }

    #[test]
    fn new_creates_empty_manager() {
        let (manager, ..) = make_manager();
        assert!(manager.status().is_empty());
        assert!(!manager.is_running(7171));
    }

    #[test]
    fn status_after_spawn_server() {
        let (mut manager, ..) = make_manager();
        let result = manager.spawn_server(dummy_settings());
        assert!(result.is_ok());
        assert!(!manager.status().is_empty());
    }

    #[test]
    fn spawn_duplicate_port_returns_error() {
        let (mut manager, ..) = make_manager();
        let mut cfg = dummy_settings();
        cfg.port = 9999;
        assert!(manager.spawn_server(cfg.clone()).is_ok());

        let result = manager.spawn_server(cfg);
        assert!(matches!(result, Err(NetworkError::AlreadyRunning(9999))));
    }

    #[test]
    fn is_running_returns_true_for_active_port() {
        let (mut manager, ..) = make_manager();
        let mut cfg = dummy_settings();
        cfg.port = 8888;
        manager
            .spawn_server(cfg)
            .expect("test server spawn should succeed");
        assert!(manager.is_running(8888));
    }

    #[test]
    fn is_running_returns_false_for_unknown_port() {
        let (manager, ..) = make_manager();
        assert!(!manager.is_running(9999));
    }

    #[test]
    fn stop_returns_ok_for_running_server() {
        let (mut manager, ..) = make_manager();
        let mut cfg = dummy_settings();
        cfg.port = 7777;
        manager
            .spawn_server(cfg)
            .expect("test server spawn should succeed");
        assert!(manager.stop(7777).is_ok());
        assert!(!manager.is_running(7777));
    }

    #[test]
    fn stop_returns_error_for_not_running() {
        let (mut manager, ..) = make_manager();
        assert!(matches!(
            manager.stop(6666),
            Err(NetworkError::NotRunning(6666))
        ));
    }

    #[test]
    fn shutdown_all_empties_all_servers() {
        let (mut manager, ..) = make_manager();
        let mut cfg1 = dummy_settings();
        cfg1.port = 10001;
        let mut cfg2 = dummy_settings();
        cfg2.port = 10002;
        manager
            .spawn_server(cfg1)
            .expect("test server spawn should succeed");
        manager
            .spawn_server(cfg2)
            .expect("test server spawn should succeed");
        assert_eq!(manager.status().len(), 2);
        manager.shutdown_all();
        assert!(manager.status().is_empty());
    }

    #[test]
    fn restart_stop_and_spawn_server() {
        let (mut manager, ..) = make_manager();
        let mut cfg = dummy_settings();
        cfg.port = 5555;
        manager
            .spawn_server(cfg.clone())
            .expect("test server spawn should succeed");
        assert!(manager.is_running(5555));
        assert!(manager.restart(&cfg).is_ok());
    }
}
