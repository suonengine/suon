use std::sync::Arc;
use tracing::info;

use suon_channel::BufferPool;
use tokio::net::TcpListener;

use crate::{
    connection::manager::ConnectionManager,
    server::{
        http::acceptor::HttpAcceptor, kind::ServerKind, settings::ServerSettings,
        shutdown::Shutdown, tcp::acceptor::TcpAcceptor,
    },
};

pub(crate) struct BoundServer {
    listener: TcpListener,
    channel: suon_channel::Channel,
    buffer_pool: Arc<BufferPool>,
    connection_manager: Arc<ConnectionManager>,
    settings: ServerSettings,
    shutdown: Shutdown,
}

impl BoundServer {
    pub fn new(
        listener: TcpListener,
        channel: suon_channel::Channel,
        settings: ServerSettings,
        shutdown: Shutdown,
        buffer_pool: Arc<BufferPool>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        BoundServer {
            listener,
            channel,
            buffer_pool,
            connection_manager,
            settings,
            shutdown,
        }
    }

    pub fn into_server(self) -> ActiveServer {
        info!(
            target: "Server",
            "Starting {} server on port {}",
            self.settings.kind.as_str(),
            self.settings.port
        );

        match self.settings.kind {
            ServerKind::Tcp { .. } => ActiveServer::Tcp(TcpAcceptor::new(
                self.listener,
                self.channel,
                &self.settings,
                self.shutdown,
                self.buffer_pool,
                self.connection_manager,
            )),
            ServerKind::Http { .. } => ActiveServer::Http(HttpAcceptor::new(
                self.listener,
                self.channel,
                &self.settings,
                self.shutdown,
            )),
        }
    }
}

pub(crate) enum ActiveServer {
    Tcp(TcpAcceptor),
    Http(HttpAcceptor),
}

impl ActiveServer {
    pub fn spawn(self) {
        match self {
            ActiveServer::Tcp(acceptor) => acceptor.spawn(),
            ActiveServer::Http(acceptor) => acceptor.spawn(),
        }
    }
}

#[cfg(test)]
mod bound_server_tests {
    use super::*;
    use crate::server::{
        kind::ServerKind,
        settings::ServerSettings,
        tcp::{EncryptionSettings, ProtocolSettings},
    };
    use std::time::Duration;

    fn test_tcp_settings() -> ServerSettings {
        ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings {
                    header_size: 2,
                    has_checksum: true,
                    uses_xtea: false,
                    uses_rsa: false,
                },
                flush_interval: Duration::from_millis(50),
                encryption: EncryptionSettings {
                    incoming: false,
                    outgoing: false,
                },
                channel_capacity: 16,
                max_buffer_size: 256,
                max_connections: 5,
                rate_burst: 50,
            },
            retry_delay: Duration::from_millis(100),
        }
    }

    fn test_http_settings() -> ServerSettings {
        ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Http {
                max_connections: 100,
                rate_burst: 50,
                max_headers: 32,
            },
            retry_delay: Duration::from_millis(100),
        }
    }

    fn test_manager() -> Arc<ConnectionManager> {
        Arc::new(ConnectionManager::new(0))
    }

    #[tokio::test]
    async fn bound_server_into_tcp_server_and_spawn() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for test");

        let channel = suon_channel::Channel::default();
        let shutdown = Shutdown::new();
        let settings = test_tcp_settings();

        BoundServer::new(
            listener,
            channel,
            settings,
            shutdown.clone(),
            crate::test_buffer_pool(),
            test_manager(),
        )
        .into_server()
        .spawn();

        tokio::time::sleep(Duration::from_millis(15)).await;
        shutdown.trigger();
    }

    #[tokio::test]
    async fn bound_server_into_http_server_and_spawn() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for HTTP test");

        let channel = suon_channel::Channel::default();
        let shutdown = Shutdown::new();
        let settings = test_http_settings();

        BoundServer::new(
            listener,
            channel,
            settings,
            shutdown.clone(),
            crate::test_buffer_pool(),
            test_manager(),
        )
        .into_server()
        .spawn();

        tokio::time::sleep(Duration::from_millis(15)).await;
        shutdown.trigger();
    }

    #[test]
    fn active_server_tcp_dispatch_spawn() {
        let shutdown = Shutdown::new();
        let settings = test_tcp_settings();
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("failed to bind std TCP listener for test");

        let server = std::thread::spawn(move || {
            let rt =
                tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test");

            rt.block_on(async {
                let listener = TcpListener::from_std(listener)
                    .expect("failed to convert std listener to tokio listener");

                let channel = suon_channel::Channel::default();
                let server = BoundServer::new(
                    listener,
                    channel,
                    settings,
                    shutdown.clone(),
                    crate::test_buffer_pool(),
                    test_manager(),
                )
                .into_server();

                server.spawn();

                tokio::time::sleep(Duration::from_millis(15)).await;

                shutdown.trigger();
            });
        });

        std::thread::sleep(Duration::from_millis(50));
        drop(server);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        connection::manager::ConnectionManager,
        server::{
            kind::ServerKind,
            tcp::{EncryptionSettings, ProtocolSettings},
        },
    };
    use std::{sync::Arc, time::Duration};

    fn test_settings(kind: ServerKind) -> ServerSettings {
        ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind,
            retry_delay: Duration::from_millis(100),
        }
    }

    fn test_manager() -> Arc<ConnectionManager> {
        Arc::new(ConnectionManager::new(0))
    }

    #[tokio::test]
    async fn runner_dispatches_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for dispatch test");

        let channel = suon_channel::Channel::default();
        let shutdown = Shutdown::new();

        let settings = test_settings(ServerKind::Tcp {
            protocol: ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            flush_interval: Duration::from_millis(50),
            encryption: EncryptionSettings {
                incoming: false,
                outgoing: false,
            },
            channel_capacity: 16,
            max_buffer_size: 256,
            max_connections: 5,
            rate_burst: 50,
        });

        BoundServer::new(
            listener,
            channel,
            settings,
            shutdown.clone(),
            crate::test_buffer_pool(),
            test_manager(),
        )
        .into_server()
        .spawn();

        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        shutdown.trigger();
    }

    #[tokio::test]
    async fn runner_dispatches_http() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for HTTP dispatch test");
        let channel = suon_channel::Channel::default();
        let shutdown = Shutdown::new();

        let settings = test_settings(ServerKind::Http {
            max_connections: 100,
            rate_burst: 50,
            max_headers: 32,
        });

        BoundServer::new(
            listener,
            channel,
            settings,
            shutdown.clone(),
            crate::test_buffer_pool(),
            test_manager(),
        )
        .into_server()
        .spawn();

        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        shutdown.trigger();
    }
}
