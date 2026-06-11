use std::{sync::Arc, time::Duration};

use suon_channel::Channel;
use tokio::{net::TcpListener, runtime::Runtime};

use crate::server::{runner::BoundServer, settings::ServerSettings, shutdown::Shutdown};

pub(crate) struct Binder {
    runtime: Arc<Runtime>,
    channel: Channel,
    settings: ServerSettings,
    shutdown: Shutdown,
    retry_delay: Duration,
}

impl Binder {
    pub fn new(
        runtime: Arc<Runtime>,
        channel: Channel,
        settings: ServerSettings,
        shutdown: Shutdown,
        retry_delay: Duration,
    ) -> Self {
        Binder {
            runtime,
            channel,
            settings,
            shutdown,
            retry_delay,
        }
    }

    pub fn launch(self) {
        if self.shutdown.is_triggered() {
            return;
        }

        let address = format!("{}:{}", self.settings.address, self.settings.port);

        let channel = self.channel.clone();
        let settings = self.settings.clone();
        let shutdown = self.shutdown.clone();
        let retry_delay = self.retry_delay;
        let runtime = self.runtime.clone();
        let handle = runtime.handle().clone();

        handle.spawn(async move {
            match TcpListener::bind(&address).await {
                Ok(listener) => {
                    BoundServer::new(listener, channel, settings, shutdown)
                        .into_server()
                        .spawn();
                }
                Err(e) => {
                    let kind_str = settings.kind.as_str();
                    println!(
                        "{kind_str} port {port} bind failed, scheduling retry in {retry_delay:?}: \
                         {e}",
                        port = settings.port,
                    );

                    tokio::spawn(async move {
                        tokio::time::sleep(retry_delay).await;
                        Binder::new(runtime, channel, settings, shutdown, retry_delay).launch();
                    });
                }
            }
        });
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

    #[test]
    fn binder_does_not_panic_on_launch() {
        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test"),
        );
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = dummy_settings();

        Binder::new(
            runtime,
            channel,
            settings,
            shutdown,
            Duration::from_millis(100),
        )
        .launch();
        // No panic means success
    }

    #[test]
    fn binder_skips_launch_if_triggered() {
        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test"),
        );
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = dummy_settings();
        shutdown.trigger();

        Binder::new(
            runtime,
            channel,
            settings,
            shutdown,
            Duration::from_millis(100),
        )
        .launch();
        // Should return immediately without spawning
    }

    #[test]
    fn binder_launch_with_http_settings() {
        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test"),
        );
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = ServerSettings {
            port: 0,
            address: "127.0.0.1".into(),
            kind: ServerKind::Http {
                max_connections: 100,
                rate_burst: 50,
                max_headers: 32,
            },
            retry_delay_ms: 100,
        };

        Binder::new(
            runtime,
            channel,
            settings,
            shutdown,
            Duration::from_millis(100),
        )
        .launch();
    }

    #[test]
    fn binder_retries_on_bind_failure() {
        let occupied =
            std::net::TcpListener::bind("127.0.0.1:9999").expect("failed to occupy port for test");

        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test"),
        );
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = ServerSettings {
            port: 9999,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings::default(),
                flush_interval_ms: 10,
                encryption: EncryptionSettings::default(),
                channel_capacity: 16,
                max_buffer_size: 256,
                max_connections: 5,
            },
            retry_delay_ms: 50,
        };

        // Should not panic — logs bind error and schedules retry
        Binder::new(
            runtime,
            channel,
            settings,
            shutdown,
            Duration::from_millis(50),
        )
        .launch();
        std::thread::sleep(Duration::from_millis(100));
        drop(occupied);
    }

    #[test]
    fn binder_retry_succeeds_after_port_freed() {
        let occupied =
            std::net::TcpListener::bind("127.0.0.1:9898").expect("failed to occupy port for test");

        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("failed to create tokio runtime for test"),
        );
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let settings = ServerSettings {
            port: 9898,
            address: "127.0.0.1".into(),
            kind: ServerKind::Tcp {
                protocol: ProtocolSettings::default(),
                flush_interval_ms: 10,
                encryption: EncryptionSettings::default(),
                channel_capacity: 16,
                max_buffer_size: 256,
                max_connections: 5,
            },
            retry_delay_ms: 50,
        };

        Binder::new(
            runtime.clone(),
            channel.clone(),
            settings,
            shutdown.clone(),
            Duration::from_millis(50),
        )
        .launch();
        // Let it try to bind and fail
        std::thread::sleep(Duration::from_millis(30));
        // Free the port
        drop(occupied);
        // Give time for retry to succeed — should not panic
        std::thread::sleep(Duration::from_millis(100));
    }
}
