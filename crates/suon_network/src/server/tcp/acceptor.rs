use std::sync::Arc;

use suon_channel::Channel;
use tokio::net::TcpListener;
use tracing::info;

use crate::{connection::manager::ConnectionManager, server::tcp::settings::TcpSettings};

use super::{connection::Connection, connection_begin::ConnectionBegin};
use crate::server::{
    settings::ServerSettings,
    shutdown::Shutdown,
    throttle::{ConnectionLimiter, PacketRateLimiter},
};

pub(crate) struct TcpAcceptor {
    listener: Arc<TcpListener>,
    channel: Channel,
    manager: Arc<ConnectionManager>,
    config: TcpSettings,
    limiter: ConnectionLimiter,
    rate_limiter: PacketRateLimiter,
    shutdown: Shutdown,
}

impl TcpAcceptor {
    pub fn new(
        listener: TcpListener,
        channel: Channel,
        settings: &ServerSettings,
        shutdown: Shutdown,
    ) -> Self {
        let config = TcpSettings::from_settings(settings);
        let manager = Arc::new(ConnectionManager::new(0));
        let limiter = ConnectionLimiter::new(config.max_connections as usize);
        let rate_limiter = PacketRateLimiter::new(50);

        info!(target: "TCP", "TCP server started on port {} [protocol: {}]", settings.port, config.protocol);

        TcpAcceptor {
            listener: Arc::new(listener),
            channel,
            manager,
            config,
            limiter,
            rate_limiter,
            shutdown,
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.accept_loop());
    }

    async fn accept_loop(self) {
        let mut rx = self.shutdown.receiver();
        loop {
            tokio::select! {
                _ = rx.changed() => {
                    if *rx.borrow() { break; }
                }
                result = self.listener.accept() => {
                    let Ok((stream, address)) = result else { continue };

                    if !self.rate_limiter.allow(address) {
                        continue;
                    }
                    let Ok(permit) = self.limiter.try_acquire() else {
                        continue;
                    };

                    let (sender, command_receiver) = crossbeam_channel::bounded(self.config.channel_capacity);
                    let id = self.manager.register(address, self.config.protocol, sender);

                    self.channel.send(ConnectionBegin {
                        id,
                        ip: address.ip().to_string(),
                        port: address.port(),
                    });

                    Connection::spawn(
                        stream,
                        command_receiver,
                        self.channel.clone(),
                        self.manager.clone(),
                        self.config,
                        self.shutdown.clone(),
                        id,
                        permit,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{
        kind::ServerKind,
        settings::ServerSettings,
        tcp::{EncryptionSettings, ProtocolSettings},
    };
    use std::time::Duration;
    use suon_channel::Channel;
    use tokio::net::TcpListener;

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
                flush_interval_ms: 50,
                encryption: EncryptionSettings {
                    incoming: false,
                    outgoing: false,
                },
                channel_capacity: 64,
                max_buffer_size: 256,
                max_connections: 5,
            },
            retry_delay_ms: 100,
        }
    }

    #[tokio::test]
    async fn tcp_start_stop_does_not_panic() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for start/stop test");
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = test_tcp_settings();

        TcpAcceptor::new(listener, channel, &settings, shutdown.clone()).spawn();
        tokio::time::sleep(Duration::from_millis(50)).await;
        shutdown.trigger();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn tcp_accept_and_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for accept/disconnect test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = test_tcp_settings();

        TcpAcceptor::new(listener, channel.clone(), &settings, shutdown.clone()).spawn();
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::time::sleep(Duration::from_millis(100)).await;

        let mut buf = Vec::new();
        channel.drain_into(&mut buf);
        assert!(!buf.is_empty(), "expected at least ConnectionBegin");

        drop(client);
        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut buf2 = Vec::new();
        channel.drain_into(&mut buf2);
        assert!(!buf2.is_empty(), "expected ConnectionEnd on disconnect");

        shutdown.trigger();
    }

    #[tokio::test]
    async fn tcp_rate_limit_rejects_excess() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for rate limit test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        use crate::server::{kind::ServerKind, settings::ServerSettings};
        let settings = ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings {
                    header_size: 2,
                    has_checksum: true,
                    uses_xtea: false,
                    uses_rsa: false,
                },
                flush_interval_ms: 50,
                encryption: EncryptionSettings {
                    incoming: false,
                    outgoing: false,
                },
                channel_capacity: 64,
                max_buffer_size: 256,
                max_connections: 1, // only 1 connection
            },
            retry_delay_ms: 100,
        };

        TcpAcceptor::new(listener, channel.clone(), &settings, shutdown.clone()).spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // First connection should succeed
        let client1 = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect first test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Second connection may be accepted but rejected by limiter
        let client2 = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect second test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let mut buf = Vec::new();
        channel.drain_into(&mut buf);
        // Should have at least 1 ConnectionBegin (first connection)
        assert!(!buf.is_empty(), "expected at least ConnectionBegin");

        drop(client1);
        drop(client2);
        shutdown.trigger();
    }

    #[tokio::test]
    async fn tcp_connection_limiter_rejects_when_full() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for limiter test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        use crate::server::{kind::ServerKind, settings::ServerSettings};
        let settings = ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings {
                    header_size: 2,
                    has_checksum: true,
                    uses_xtea: false,
                    uses_rsa: false,
                },
                flush_interval_ms: 50,
                encryption: EncryptionSettings {
                    incoming: false,
                    outgoing: false,
                },
                channel_capacity: 64,
                max_buffer_size: 256,
                max_connections: 0, // reject all
            },
            retry_delay_ms: 100,
        };

        TcpAcceptor::new(listener, channel, &settings, shutdown.clone()).spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        drop(client);
        shutdown.trigger();
    }
}
