use std::time::Duration;

use crossbeam_channel::TryRecvError;
use suon_network::{
    connection::{ConnectionHandle, ConnectionId},
    protocol::{Command as TcpCommand, PacketReader, PacketWriter},
    server::tcp::ProtocolSettings,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

fn status_settings() -> ProtocolSettings {
    ProtocolSettings {
        header_size: 2,
        has_checksum: true,
        uses_xtea: false,
        uses_rsa: false,
    }
}

fn game_settings() -> ProtocolSettings {
    ProtocolSettings {
        header_size: 6,
        has_checksum: true,
        uses_xtea: true,
        uses_rsa: true,
    }
}

#[test]
fn connection_handle_send_receive() {
    let (tx, rx) = crossbeam_channel::bounded(16);
    let handle = ConnectionHandle::new(
        ConnectionId::new(0, 1),
        "127.0.0.1:7000".parse().expect("test address should parse"),
        tx,
    );

    handle
        .send(vec![0xAA, 0xBB])
        .expect("failed to send test command");
    let cmd = rx.try_recv().expect("failed to receive test command");
    assert!(matches!(cmd, TcpCommand::Send(data) if data == vec![0xAA, 0xBB]));
}

#[test]
fn connection_handle_close() {
    let (tx, rx) = crossbeam_channel::bounded(16);
    let handle = ConnectionHandle::new(
        ConnectionId::new(0, 1),
        "127.0.0.1:7000".parse().expect("test address should parse"),
        tx,
    );

    handle.close().expect("failed to close handle in test");
    assert!(matches!(
        rx.try_recv().expect("failed to receive Close command"),
        TcpCommand::Close
    ));
}

#[test]
fn connection_handle_backpressure() {
    let (tx, _rx) = crossbeam_channel::bounded(2);
    let handle = ConnectionHandle::new(
        ConnectionId::new(0, 1),
        "127.0.0.1:7000".parse().expect("test address should parse"),
        tx,
    );

    handle
        .send(vec![1])
        .expect("failed to send first command in backpressure test");
    handle
        .send(vec![2])
        .expect("failed to send second command in backpressure test");
    assert!(handle.send(vec![3]).is_err());
}

#[test]
fn packet_writer_status_framing() {
    let mut writer = PacketWriter::new(status_settings());
    writer.send(b"hello");

    let buf = writer.take_buffer();
    // Frame: [body_size(2)][checksum(4)][data]
    assert_eq!(buf.len(), 2 + 4 + 5);

    let body_size = u16::from_le_bytes([buf[0], buf[1]]) as usize;
    assert_eq!(body_size, 4 + 5); // checksum(4) + data(5)

    let data = &buf[6..];
    assert_eq!(data, b"hello");
}

#[test]
fn packet_reader_status_passthrough() {
    let mut reader = PacketReader::new(status_settings());
    // Status has_checksum=true, provide data with leading checksum
    let data = b"hello";
    let checksum = suon_adler32::generate(data);
    let mut body = Vec::with_capacity(4 + data.len());
    body.extend_from_slice(&checksum.to_le_bytes());
    body.extend_from_slice(data);
    let result = reader
        .process(&body)
        .expect("reader should process status checksum body");
    assert_eq!(result, Some(data.to_vec()));
}

#[test]
fn packet_reader_writer_xtea_roundtrip() {
    let key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
    let mut writer = PacketWriter::new(game_settings());
    writer.set_xtea_key(key);
    writer.set_xtea_enabled(true);
    writer.send(b"secret data");
    let framed = writer.take_buffer();

    // Strip the 2-byte size header before passing to reader
    let body = &framed[2..];

    let mut reader = PacketReader::new(game_settings());
    reader.set_xtea_key(key);
    reader.set_xtea_enabled(true);
    reader.set_rsa_done(true);
    let result = reader
        .process(body)
        .expect("reader should process XTEA roundtrip");
    assert_eq!(result, Some(b"secret data".to_vec()));
}

#[test]
fn packet_writer_includes_checksum() {
    let mut writer = PacketWriter::new(status_settings());
    writer.send(b"hello");
    let framed = writer.take_buffer();

    // Status frame layout: [size(2)][checksum(4)][data]
    assert!(framed.len() >= 6);
    let checksum_bytes = &framed[2..6];
    let data = &framed[6..];
    assert_eq!(data, b"hello");
    let expected = suon_adler32::generate(b"hello").to_le_bytes();
    assert_eq!(checksum_bytes, &expected);
}

#[test]
fn packet_writer_buffer_accumulation() {
    let mut writer = PacketWriter::new(status_settings());
    writer.send(b"a");
    let len1 = writer.buffer_len();
    writer.send(b"b");
    let len2 = writer.buffer_len();
    assert!(len2 > len1);
}

#[test]
fn packet_writer_flush_by_size() {
    let mut writer = PacketWriter::new(status_settings());
    writer.set_max_buffer_size(32);
    writer.send(b"hello");
    assert!(!writer.should_flush_by_size());
    writer.send(b"world");
    assert!(!writer.should_flush_by_size());

    let buf = writer.take_buffer();
    assert!(buf.len() >= 12);
    assert!(writer.is_empty());
}

#[test]
fn adler32_consistency() {
    let data = b"test data for checksum";
    assert_eq!(suon_adler32::generate(data), suon_adler32::generate(data));
}

#[test]
fn xtea_pad_unpad_roundtrip() {
    for len in 0..32 {
        let data = vec![0xABu8; len];
        let padded = suon_network::server::tcp::xtea_pad(&data);
        let unpadded = suon_network::server::tcp::xtea_unpad(&padded);
        assert_eq!(unpadded, data.as_slice(), "len={len}");
    }
}

#[tokio::test]
async fn tcp_status_echo_roundtrip() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind listener for echo test");
    let addr = listener
        .local_addr()
        .expect("failed to get listener local address");
    let proto = status_settings();

    tokio::spawn(async move {
        let (mut stream, _) = listener
            .accept()
            .await
            .expect("failed to accept connection in echo test");
        let mut buf = vec![0u8; 1024];
        let n = stream
            .read(&mut buf)
            .await
            .expect("failed to read from stream in echo test");
        buf.truncate(n);
        stream
            .write_all(&buf)
            .await
            .expect("failed to write to stream in echo test");
        stream
            .flush()
            .await
            .expect("failed to flush stream in echo test");
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .expect("failed to connect client in echo test");
    let mut writer = PacketWriter::new(proto);
    let payload = b"ping";
    writer.send(payload);
    let framed = writer.take_buffer();
    client
        .write_all(&framed)
        .await
        .expect("failed to write framed data in echo test");
    client
        .flush()
        .await
        .expect("failed to flush client in echo test");

    let mut reader = PacketReader::new(proto);
    let mut buf = [0u8; 1024];
    let n = client
        .read(&mut buf)
        .await
        .expect("failed to read response in echo test");
    let response = &buf[..n];

    // Strip the 2-byte size header before passing to reader
    let body = &response[2..];
    let result = reader
        .process(body)
        .expect("reader should process echoed data");
    assert_eq!(result, Some(payload.to_vec()));
}

#[tokio::test]
async fn tcp_multiple_connections() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind listener for multi-connection test");
    let addr = listener
        .local_addr()
        .expect("failed to get listener local address");
    let proto = status_settings();

    let accept_task = tokio::spawn(async move {
        for i in 0..3 {
            let (mut stream, _) = listener
                .accept()
                .await
                .expect("failed to accept connection in multi test");
            let mut buf = vec![0u8; 1024];
            let n = stream
                .read(&mut buf)
                .await
                .expect("failed to read from stream in multi test");
            buf.truncate(n);
            let payload = format!("response-{i}");
            let mut w = PacketWriter::new(proto);
            w.send(payload.as_bytes());
            let framed = w.take_buffer();
            stream
                .write_all(&framed)
                .await
                .expect("failed to write response in multi test");
            stream
                .flush()
                .await
                .expect("failed to flush stream in multi test");
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    for i in 0..3 {
        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect client in multi test");
        let mut w = PacketWriter::new(proto);
        let payload = format!("hello-{i}");
        w.send(payload.as_bytes());
        client
            .write_all(&w.take_buffer())
            .await
            .expect("failed to write payload in multi test");
        client
            .flush()
            .await
            .expect("failed to flush client in multi test");

        let mut r = PacketReader::new(proto);
        let mut buf = [0u8; 1024];
        let n = client
            .read(&mut buf)
            .await
            .expect("failed to read response in multi test");
        let body = &buf[2..n];
        let result = r
            .process(body)
            .expect("reader should process multi-connection data");
        let expected = format!("response-{i}");
        assert_eq!(result, Some(expected.into_bytes()));
    }

    accept_task
        .await
        .expect("accept task should complete successfully");
}

#[tokio::test]
async fn tcp_large_payload_roundtrip() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind listener for large payload test");
    let addr = listener
        .local_addr()
        .expect("failed to get listener local address");
    let proto = status_settings();

    tokio::spawn(async move {
        let (mut stream, _) = listener
            .accept()
            .await
            .expect("failed to accept connection in large payload test");
        let mut buf = vec![0u8; 4096];
        let n = stream
            .read(&mut buf)
            .await
            .expect("failed to read in large payload test");
        buf.truncate(n);
        stream
            .write_all(&buf)
            .await
            .expect("failed to write in large payload test");
        stream
            .flush()
            .await
            .expect("failed to flush in large payload test");
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .expect("failed to connect client in large payload test");
    let mut writer = PacketWriter::new(proto);
    let payload = vec![0xABu8; 1500];
    writer.send(&payload);
    client
        .write_all(&writer.take_buffer())
        .await
        .expect("failed to write large payload");
    client
        .flush()
        .await
        .expect("failed to flush large payload client");

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut reader = PacketReader::new(proto);
    let mut buf = [0u8; 4096];
    let n = client
        .read(&mut buf)
        .await
        .expect("failed to read response in large payload test");
    let body = &buf[2..n];
    let result = reader
        .process(body)
        .expect("reader should process large payload data");
    assert_eq!(result, Some(payload));
}

#[test]
fn tcp_connection_drop_cleanup() {
    let (tx, rx) = crossbeam_channel::bounded::<TcpCommand>(16);
    let handle = ConnectionHandle::new(
        ConnectionId::new(0, 1),
        "127.0.0.1:0".parse().expect("test address should parse"),
        tx,
    );
    drop(handle);

    let result = rx.try_recv();
    assert!(matches!(result, Err(TryRecvError::Disconnected)));
}
