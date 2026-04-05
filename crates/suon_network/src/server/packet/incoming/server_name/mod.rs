use bevy::{
    prelude::*,
    tasks::futures_lite::{AsyncRead, AsyncReadExt},
};
use thiserror::Error;

use crate::server::packet::incoming::{IncomingPacket, server_name::buffer::PacketBuffer};

mod buffer;

/// Errors that can occur while reading or decoding a server name packet.
///
/// These errors cover I/O failures, truncated or malformed data and cases
/// where the received buffer exceeds protocol-defined limits.
#[derive(Debug, Error)]
pub(crate) enum PacketReadError {
    /// The connection was closed before a complete packet could be read.
    ///
    /// This typically indicates that the remote peer disconnected or reset
    /// the connection mid-transmission.
    #[error("connection closed before reading complete packet")]
    ConnectionClosed,

    /// An I/O error occurred while reading from the socket.
    ///
    /// Wraps a standard [`std::io::Error`], typically indicating a network
    /// failure or an unexpected stream termination.
    #[error("I/O error while reading packet: {0}")]
    Io(#[from] std::io::Error),

    /// The accumulated buffer exceeded the maximum allowed size.
    ///
    /// This usually points to a malformed or malicious packet that does not
    /// include a valid terminator, causing unbounded growth.
    #[error("packet size ({buffer_len} bytes) exceeds maximum allowed size ({max} bytes)")]
    LengthOutOfBounds {
        /// Maximum allowed packet size.
        max: usize,
        /// Actual buffer size when overflow occurred.
        buffer_len: usize,
    },

    /// The packet did not contain the expected newline (`\n`) terminator.
    ///
    /// This indicates incomplete or corrupted data, possibly truncated
    /// during transmission.
    #[error("packet missing newline terminator")]
    MissingTerminator,
}

/// Asynchronous trait for reading and decoding a server name packet from a stream.
///
/// This trait provides an extension method for any type implementing
/// [`AsyncRead`], enabling it to read a single server name packet
/// in accordance with the protocol definition.
pub(crate) trait ServerNameReadPacketExt {
    /// Reads and decodes a single server name packet from the underlying stream.
    fn read_server_name_packet(
        &mut self,
        max_length: usize,
    ) -> impl Future<Output = Result<IncomingPacket, PacketReadError>>;
}

impl<T> ServerNameReadPacketExt for T
where
    T: AsyncRead + Unpin + Send + Sync,
{
    async fn read_server_name_packet(
        &mut self,
        max_length: usize,
    ) -> Result<IncomingPacket, PacketReadError> {
        trace!("Starting to read server name packet");

        // Initialize a buffer for accumulating incoming bytes
        let mut buffer = PacketBuffer::with_capacity(max_length);

        // Read bytes from the socket into the buffer
        let n = self.read(buffer.payload_mut()).await.map_err(|err| {
            warn!("I/O error while reading from socket: {:?}", err);
            PacketReadError::Io(err)
        })?;

        trace!("Read {} bytes from socket", n);

        // If zero bytes read, the connection was closed
        if n == 0 {
            warn!("Connection closed while reading server name packet");
            return Err(PacketReadError::ConnectionClosed);
        }

        // Truncate the internal buffer to match the number of bytes read
        buffer.truncate(n);

        let len = buffer.payload_len();
        trace!("Current buffer length: {}", len);

        // Ensure the accumulated buffer does not exceed the maximum allowed length
        if len > max_length {
            warn!(
                "Buffer exceeded maximum packet size: {} > {}",
                len, max_length
            );

            return Err(PacketReadError::LengthOutOfBounds {
                max: max_length,
                buffer_len: len,
            });
        }

        // Attempt to extract a complete packet from the buffer
        match buffer.take_packet() {
            Some(packet) => {
                trace!("Successfully extracted server name packet from buffer");
                Ok(packet)
            }
            None => {
                // If buffer reached maximum length but no newline found, the packet is malformed
                warn!("Buffer reached maximum length but packet incomplete");
                Err(PacketReadError::MissingTerminator)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::tasks::futures_lite::io::Cursor;
    use smol::block_on;
    use suon_protocol::packets::client::PacketKind;

    #[test]
    fn should_return_connection_closed_when_the_stream_is_empty() {
        let mut reader = Cursor::new(Vec::<u8>::new());

        let error = block_on(reader.read_server_name_packet(32))
            .expect_err("Empty streams should be treated as closed connections");

        assert!(matches!(error, PacketReadError::ConnectionClosed));
    }

    #[test]
    fn should_decode_a_newline_terminated_server_name_packet() {
        let mut reader = Cursor::new(b"suon\n".to_vec());

        let packet = block_on(reader.read_server_name_packet(32))
            .expect("A newline-terminated payload should decode as a server-name packet");

        assert_eq!(
            packet.kind,
            PacketKind::ServerName,
            "Server-name packet reads should tag the decoded packet correctly"
        );
        assert_eq!(
            &packet.buffer[..2],
            &[4, 0],
            "Server-name packet reads should prepend the encoded payload length"
        );
        assert_eq!(
            &packet.buffer[2..],
            b"suon",
            "Server-name packet reads should strip the newline terminator from the payload"
        );
    }
}
