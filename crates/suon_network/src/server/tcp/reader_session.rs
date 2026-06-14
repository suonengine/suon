use std::sync::Arc;
use tracing::{error, trace};

use suon_channel::{BufferPool, Channel};
use tokio::io::AsyncReadExt;

use crate::{
    connection::{id::ConnectionId, manager::ConnectionManager},
    protocol::reader::{PacketReader, ProcessOutcome},
    server::tcp::settings::TcpSettings,
};

use super::{connection_end::ConnectionEnd, raw_packet::RawPacket};
use crate::server::{shutdown::Shutdown, throttle::ConnectionPermit};

pub(crate) struct ReaderSession {
    id: ConnectionId,
    reader_half: tokio::net::tcp::OwnedReadHalf,
    reader_channel: Channel,
    buffer_pool: Arc<BufferPool>,
    config: TcpSettings,
    shutdown: Shutdown,
    manager: Arc<ConnectionManager>,
    permit: Option<ConnectionPermit>,
}

impl ReaderSession {
    pub fn new(
        id: ConnectionId,
        reader_half: tokio::net::tcp::OwnedReadHalf,
        reader_channel: Channel,
        config: TcpSettings,
        shutdown: Shutdown,
        manager: Arc<ConnectionManager>,
        permit: ConnectionPermit,
        buffer_pool: Arc<BufferPool>,
    ) -> Self {
        ReaderSession {
            id,
            reader_half,
            reader_channel,
            buffer_pool,
            config,
            shutdown,
            manager,
            permit: Some(permit),
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        let mut reader = PacketReader::new(self.config.protocol);
        reader.set_xtea_enabled(self.config.encryption.incoming);

        let mut size_buf = [0u8; 2];
        let mut body_buf = self.buffer_pool.acquire();
        let mut rx = self.shutdown.receiver();
        trace!(target: "TCP", "Reader session {} started", self.id);

        loop {
            let size = tokio::select! {
                _ = rx.changed() => {
                    if *rx.borrow() { break; }
                    continue;
                }
                result = self.reader_half.read(&mut size_buf) => {
                    match result {
                        Ok(2) => u16::from_le_bytes(size_buf) as usize,
                        _ => break,
                    }
                }
            };

            if size == 0 {
                continue;
            }

            body_buf.resize(size, 0);
            let body_slice = &mut body_buf[..size];

            tokio::select! {
                _ = rx.changed() => {
                    if *rx.borrow() { break; }
                }
                result = self.reader_half.read_exact(body_slice) => {
                    if result.is_err() { break; }
                }
            }

            trace!(target: "TCP", "Reader session {} processing {} bytes", self.id, size);
            match reader.process_in_place(&mut body_buf) {
                Ok(ProcessOutcome::Complete) => {
                    let data = std::mem::take(&mut body_buf);
                    self.reader_channel.send(RawPacket { id: self.id, data });
                    body_buf = self.buffer_pool.acquire();
                }
                Ok(ProcessOutcome::Skip) => {}
                Err(e) => {
                    error!(target: "TCP", "Reader session {} processing error: {e}", self.id);
                    break;
                }
            }
        }

        self.buffer_pool.release(body_buf);
        self.reader_channel.send(ConnectionEnd { id: self.id });
        self.manager.unregister(self.id);
        drop(self.permit.take());
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

    fn setup() -> (Arc<ConnectionManager>, ConnectionPermit) {
        let manager = Arc::new(ConnectionManager::new(0));
        let limiter = ConnectionLimiter::new(5);
        let permit = limiter
            .try_acquire()
            .expect("failed to acquire connection permit for test");

        (manager, permit)
    }

    #[tokio::test]
    async fn reader_session_spawn_and_cleanup_on_disconnect() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for cleanup test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let (manager, permit) = setup();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");

            let (reader_half, ..) = stream.into_split();
            let (sender, ..) = crossbeam_channel::bounded(64);
            let id = manager.register(addr, config.protocol, sender);

            ReaderSession::new(
                id,
                reader_half,
                channel,
                config,
                shutdown,
                manager,
                permit,
                crate::test_buffer_pool(),
            )
            .spawn();
        });

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        drop(client);
        drop(server.await);
    }

    #[tokio::test]
    async fn reader_session_exits_on_eof() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for EOF test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let (manager, permit) = setup();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");

            let (reader_half, ..) = stream.into_split();
            let (sender, ..) = crossbeam_channel::bounded(64);
            let id = manager.register(addr, config.protocol, sender);

            ReaderSession::new(
                id,
                reader_half,
                channel,
                config,
                shutdown,
                manager,
                permit,
                crate::test_buffer_pool(),
            )
            .spawn();
        });

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        drop(client);
        drop(server.await);
    }

    #[tokio::test]
    async fn reader_session_exits_on_partial_read() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for partial read test");

        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");

        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let (manager, permit) = setup();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");

            let (reader_half, ..) = stream.into_split();
            let (sender, ..) = crossbeam_channel::bounded(64);
            let id = manager.register(addr, config.protocol, sender);

            ReaderSession::new(
                id,
                reader_half,
                channel,
                config,
                shutdown,
                manager,
                permit,
                crate::test_buffer_pool(),
            )
            .spawn();
        });

        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        use tokio::io::AsyncWriteExt;
        client
            .write_all(b"\x00\x05")
            .await
            .expect("failed to write partial data in test");

        client.flush().await.expect("failed to flush test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        drop(client);
        drop(server.await);
    }
}
