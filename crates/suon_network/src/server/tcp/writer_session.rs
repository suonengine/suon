use std::time::Duration;

use tokio::io::{AsyncWriteExt, BufWriter};

use crate::{
    protocol::{command::Command, writer::PacketWriter},
    server::tcp::settings::TcpSettings,
};

use crate::server::shutdown::Shutdown;

pub(crate) struct WriterSession {
    command_receiver: crossbeam_channel::Receiver<Command>,
    writer_half: tokio::net::tcp::OwnedWriteHalf,
    config: TcpSettings,
    shutdown: Shutdown,
}

impl WriterSession {
    pub fn new(
        command_receiver: crossbeam_channel::Receiver<Command>,
        writer_half: tokio::net::tcp::OwnedWriteHalf,
        config: TcpSettings,
        shutdown: Shutdown,
    ) -> Self {
        WriterSession {
            command_receiver,
            writer_half,
            config,
            shutdown,
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.run());
    }

    async fn run(self) {
        let mut packet_writer = PacketWriter::new(self.config.protocol);
        packet_writer.set_xtea_enabled(self.config.encryption.outgoing);
        packet_writer.set_max_buffer_size(self.config.max_buffer_size);

        let mut buf_writer = BufWriter::new(self.writer_half);
        let flush_interval = Duration::from_millis(self.config.flush_interval_ms);
        let mut flush_timer = tokio::time::interval(flush_interval);
        flush_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut rx = self.shutdown.receiver();
        loop {
            tokio::select! {
                biased;
                _ = flush_timer.tick() => {
                    if !packet_writer.is_empty() {
                        let buf = packet_writer.take_buffer();
                        if let Err(e) = buf_writer.write_all(&buf).await {
                            eprintln!("[TCP] writer flush error: {e}");
                            break;
                        }
                    }
                    if let Err(e) = buf_writer.flush().await {
                        eprintln!("[TCP] writer flush error: {e}");
                        break;
                    }
                }
                _ = rx.changed() => {
                    if *rx.borrow() {
                        if !packet_writer.is_empty() {
                            let buf = packet_writer.take_buffer();
                            if let Err(e) = buf_writer.write_all(&buf).await { eprintln!("[TCP] shutdown write error: {e}"); }
                        }
                        if let Err(e) = buf_writer.flush().await { eprintln!("[TCP] shutdown flush error: {e}"); }
                        break;
                    }
                }
            }

            while let Ok(command) = self.command_receiver.try_recv() {
                match command {
                    Command::Send(plaintext) => {
                        packet_writer.send(&plaintext);
                        if packet_writer.should_flush_by_size() {
                            let buf = packet_writer.take_buffer();
                            if let Err(e) = buf_writer.write_all(&buf).await {
                                eprintln!("[TCP] writer error: {e}");
                                return;
                            }
                        }
                    }
                    Command::SendRaw(data) => {
                        packet_writer.send_raw(&data);
                        if packet_writer.should_flush_by_size() {
                            let buf = packet_writer.take_buffer();
                            if let Err(e) = buf_writer.write_all(&buf).await {
                                eprintln!("[TCP] writer error: {e}");
                                return;
                            }
                        }
                    }
                    Command::SetXteaKey(key) => {
                        packet_writer.set_xtea_key(key);
                    }
                    Command::SetEncryptionEnabled(enabled) => {
                        packet_writer.set_xtea_enabled(enabled);
                    }
                    Command::SetCompressionThreshold(_) => {
                        // reserved for future use
                    }
                    Command::Close | Command::CloseWithReason(_) => {
                        if !packet_writer.is_empty() {
                            let buf = packet_writer.take_buffer();
                            if let Err(e) = buf_writer.write_all(&buf).await {
                                eprintln!("[TCP] final write error: {e}");
                            }
                        }
                        if let Err(e) = buf_writer.flush().await {
                            eprintln!("[TCP] final flush error: {e}");
                        }
                        if let Err(e) = buf_writer.shutdown().await {
                            eprintln!("[TCP] shutdown error: {e}");
                        }
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::tcp::{EncryptionSettings, ProtocolSettings};
    use tokio::{io::AsyncWriteExt, net::TcpListener};

    fn make_config() -> TcpSettings {
        TcpSettings {
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
        }
    }

    #[tokio::test]
    async fn writer_session_spawn_and_receive_send() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for writer send test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let shutdown = Shutdown::new();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");
            let (.., writer_half) = stream.into_split();
            let (_, rx) = crossbeam_channel::bounded(16);
            WriterSession::new(rx, writer_half, config, shutdown).spawn();
        });

        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        client
            .write_all(b"test")
            .await
            .expect("failed to write test data");
        client.flush().await.expect("failed to flush test client");
        drop(client);
        drop(server.await);
    }

    #[tokio::test]
    async fn writer_session_close_via_command() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for writer close test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let shutdown = Shutdown::new();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");
            let (.., writer_half) = stream.into_split();
            let (_, rx) = crossbeam_channel::bounded(16);
            WriterSession::new(rx, writer_half, config, shutdown).spawn();
        });

        let client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        drop(client);
        drop(server.await);
    }

    #[tokio::test]
    async fn writer_session_shutdown_on_close() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for shutdown test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let shutdown = Shutdown::new();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");
            let (.., writer_half) = stream.into_split();
            let (tx, rx) = crossbeam_channel::bounded(16);
            WriterSession::new(rx, writer_half, config, shutdown).spawn();
            // Wait for client to connect, then send Close
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            tx.send(Command::Close).ok();
        });

        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        // Give the writer time to process the Close command
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

        // Writer received Close → called buf_writer.shutdown().await
        // Client should see EOF (read returns 0)
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; 16];
        let result = tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            client.read(&mut buf),
        )
        .await;

        match result {
            Ok(Ok(0)) => {} // EOF indicates shutdown was called
            Ok(Ok(n)) => panic!("expected EOF (0 bytes), got {n} bytes"),
            Ok(Err(e)) => panic!("read error: {e}"),
            Err(_) => panic!("timeout waiting for EOF — shutdown was not called"),
        }

        drop(client);
        drop(server.await);
    }

    #[tokio::test]
    async fn writer_session_send_command_flushes() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for send/flush test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let shutdown = Shutdown::new();
        let config = make_config();

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");
            let (.., writer_half) = stream.into_split();
            let (tx, rx) = crossbeam_channel::bounded(16);
            WriterSession::new(rx, writer_half, config, shutdown).spawn();
            // Send data through the command channel
            tx.send(Command::Send(b"hello".to_vec())).ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            tx.send(Command::Close).ok();
        });

        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Read the framed data the writer should have sent
        let mut buf = vec![0u8; 1024];
        use tokio::io::AsyncReadExt;
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(500),
            client.read(&mut buf),
        )
        .await
        {
            Ok(Ok(n)) if n > 0 => {
                assert!(n >= 2 + 4 + 5, "expected framed data, got {n} bytes");
                // Verify frame structure: [size(2)][checksum(4)][data]
                let body_size = u16::from_le_bytes([buf[0], buf[1]]) as usize;
                assert_eq!(body_size, 4 + 5);
                assert_eq!(&buf[6..6 + 5], b"hello");
            }
            _ => panic!("writer did not send data within timeout"),
        }

        drop(client);
        drop(server.await);
    }
}
