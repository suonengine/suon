use std::sync::Arc;

use suon_channel::Channel;
use tokio::net::TcpListener;
use tracing::info;

use super::session::HttpSession;
use crate::server::{
    settings::ServerSettings,
    shutdown::Shutdown,
    throttle::{ConnectionLimiter, PacketRateLimiter},
};

const MAX_HEADERS: usize = 64;

#[derive(Clone, Copy)]
pub(crate) struct HttpSettings {
    pub max_connections: usize,
    pub rate_burst: u32,
    pub max_headers: usize,
    pub port: u16,
}

impl HttpSettings {
    pub fn from_settings(settings: &ServerSettings) -> Self {
        match &settings.kind {
            crate::server::kind::ServerKind::Http {
                max_connections,
                rate_burst,
                max_headers,
                ..
            } => HttpSettings {
                max_connections: *max_connections as usize,
                rate_burst: *rate_burst,
                max_headers: (*max_headers).min(MAX_HEADERS),
                port: settings.port,
            },
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod http_settings_tests {
    use super::*;
    use crate::server::{kind::ServerKind, settings::ServerSettings};
    use std::time::Duration;

    #[test]
    fn http_settings_from_http_server() {
        let settings = ServerSettings {
            port: 8080,
            address: "0.0.0.0".into(),
            kind: ServerKind::Http {
                max_connections: 200,
                rate_burst: 100,
                max_headers: 64,
            },
            retry_delay: Duration::from_millis(15000),
        };
        let http = HttpSettings::from_settings(&settings);
        assert_eq!(http.max_connections, 200);
        assert_eq!(http.rate_burst, 100);
        assert_eq!(http.max_headers, 64);
        assert_eq!(http.port, 8080);
    }

    #[test]
    fn http_settings_defaults_are_reasonable() {
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
        let http = HttpSettings::from_settings(&settings);
        assert_eq!(http.max_connections, 100);
        assert_eq!(http.rate_burst, 50);
        assert_eq!(http.max_headers, 32);
    }

    #[test]
    #[should_panic(expected = "internal error: entered unreachable code")]
    fn http_settings_from_tcp_panics() {
        let settings = ServerSettings {
            port: 7171,
            address: "0.0.0.0".into(),
            kind: ServerKind::Tcp {
                protocol: crate::server::tcp::ProtocolSettings::default(),
                flush_interval: Duration::from_millis(10),
                encryption: crate::server::tcp::EncryptionSettings::default(),
                channel_capacity: 128,
                max_buffer_size: 4096,
                max_connections: 100,
                rate_burst: 50,
            },
            retry_delay: Duration::from_millis(15000),
        };
        HttpSettings::from_settings(&settings);
    }
}

pub(crate) struct HttpAcceptor {
    listener: Arc<TcpListener>,
    channel: Channel,
    config: HttpSettings,
    limiter: ConnectionLimiter,
    rate_limiter: PacketRateLimiter,
    shutdown: Shutdown,
}

impl HttpAcceptor {
    pub fn new(
        listener: TcpListener,
        channel: Channel,
        settings: &ServerSettings,
        shutdown: Shutdown,
    ) -> Self {
        let config = HttpSettings::from_settings(settings);
        let limiter = ConnectionLimiter::new(config.max_connections);
        let rate_limiter = PacketRateLimiter::new(config.rate_burst);

        info!(target: "HTTP", "HTTP server started on port {}", settings.port);

        HttpAcceptor {
            listener: Arc::new(listener),
            channel,
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
        let mut request_id: u64 = 0;

        loop {
            let mut rx = self.shutdown.receiver();
            tokio::select! {
                _ = rx.changed() => {
                    if *rx.borrow() { break; }
                }
                result = self.listener.accept() => {
                    let Ok((stream, address)) = result else {
                        continue
                    };

                    if !self.rate_limiter.allow(address) {
                        continue;
                    }

                    let Ok(permit) = self.limiter.try_acquire() else {
                        continue;
                    };

                    request_id += 1;

                    HttpSession::new(
                            request_id,
                            stream,
                            self.channel.clone(),
                            self.config,
                            self.shutdown.clone(),
                            permit,
                        )
                        .spawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod http_acceptor_tests {
    use super::*;
    use crate::server::{kind::ServerKind, settings::ServerSettings, shutdown::Shutdown};
    use std::time::Duration;
    use suon_channel::Channel;

    fn make_settings() -> ServerSettings {
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

    #[tokio::test]
    async fn http_acceptor_spawn_does_not_panic() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for spawn test");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = make_settings();

        HttpAcceptor::new(listener, channel, &settings, shutdown.clone()).spawn();
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
        shutdown.trigger();
        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;
    }

    #[tokio::test]
    async fn http_acceptor_accepts_connection() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for accept test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = make_settings();

        HttpAcceptor::new(listener, channel.clone(), &settings, shutdown.clone()).spawn();

        tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        drop(client);
        shutdown.trigger();
    }
}
