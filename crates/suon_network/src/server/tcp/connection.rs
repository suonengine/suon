use std::sync::Arc;
use tracing::trace;

use suon_channel::{BufferPool, Channel};
use tokio::net::TcpStream;

use crate::{
    connection::{id::ConnectionId, manager::ConnectionManager},
    protocol::command::Command,
    server::tcp::settings::TcpSettings,
};

use super::{reader_session::ReaderSession, writer_session::WriterSession};
use crate::server::{shutdown::Shutdown, throttle::ConnectionPermit};

pub(crate) struct Connection;

#[allow(clippy::too_many_arguments)]
impl Connection {
    pub fn spawn(
        stream: TcpStream,
        command_receiver: crossbeam_channel::Receiver<Command>,
        channel: Channel,
        manager: Arc<ConnectionManager>,
        config: TcpSettings,
        shutdown: Shutdown,
        handle_id: ConnectionId,
        permit: ConnectionPermit,
        buffer_pool: Arc<BufferPool>,
    ) {
        if let Ok(addr) = stream.peer_addr() {
            trace!(target: "Connection", "Spawning TCP connection {handle_id} from {addr}");
        }

        let (reader_half, writer_half) = stream.into_split();

        ReaderSession::new(
            handle_id,
            reader_half,
            channel,
            config,
            shutdown.clone(),
            manager,
            permit,
            buffer_pool.clone(),
        )
        .spawn();

        WriterSession::new(command_receiver, writer_half, config, shutdown, buffer_pool).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::throttle::ConnectionLimiter;
    use std::{sync::Arc, time::Duration};
    use tokio::net::TcpListener;

    fn make_config() -> TcpSettings {
        TcpSettings {
            protocol: crate::server::tcp::ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            flush_interval: Duration::from_millis(50),
            encryption: crate::server::tcp::EncryptionSettings {
                incoming: false,
                outgoing: false,
            },
            channel_capacity: 64,
            max_buffer_size: 256,
            max_connections: 5,
            connection_timeout_secs: 10,
            rate_burst: 50,
        }
    }

    #[tokio::test]
    async fn connection_spawn_does_not_panic() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let manager = Arc::new(ConnectionManager::new(0));
        let config = make_config();
        let limiter = ConnectionLimiter::new(5);

        let permit = limiter
            .try_acquire()
            .expect("failed to acquire connection permit for test");

        let accept = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");

            let (_, rx) = crossbeam_channel::bounded(16);
            Connection::spawn(
                stream,
                rx,
                channel,
                manager,
                config,
                shutdown,
                ConnectionId::new(0, 1),
                permit,
                crate::test_buffer_pool(),
            );
        });

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");

        drop(accept.await);
        drop(client);
    }

    #[tokio::test]
    async fn connection_multiple_clients() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for multi-client test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let manager = Arc::new(ConnectionManager::new(0));
        let config = make_config();
        let limiter = ConnectionLimiter::new(5);

        let accept = tokio::spawn(async move {
            for _ in 0..3 {
                let permit = limiter
                    .try_acquire()
                    .expect("failed to acquire connection permit for multi-client test");

                let (stream, _) = listener
                    .accept()
                    .await
                    .expect("failed to accept incoming connection");

                let (_, rx) = crossbeam_channel::bounded(16);
                Connection::spawn(
                    stream,
                    rx,
                    channel.clone(),
                    manager.clone(),
                    config,
                    shutdown.clone(),
                    ConnectionId::new(0, 1),
                    permit,
                    crate::test_buffer_pool(),
                );
            }
        });

        for _ in 0..3 {
            let client = tokio::net::TcpStream::connect(addr)
                .await
                .expect("failed to connect test client");

            tokio::time::sleep(tokio::time::Duration::from_millis(15)).await;

            drop(client);
        }
        drop(accept.await);
    }
}
