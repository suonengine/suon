use bevy::{
    prelude::*,
    tasks::futures_lite::{AsyncRead, AsyncReadExt},
};
use thiserror::Error;

use crate::server::packet::incoming::{IncomingPacket, login::buffer::PacketBuffer};

mod buffer;

/// Errors that can occur while reading or decoding a login packet from a client.
///
/// These errors represent all possible failure conditions that can happen
/// during the reading, validation and decoding stages of the login packet
/// exchange process.
#[derive(Debug, Error)]
pub(crate) enum PacketReadError {
    /// The connection was closed before a complete packet could be read.
    ///
    /// This usually indicates that the client disconnected unexpectedly or
    /// that the connection was reset mid-transmission.
    #[error("connection closed before the packet was fully read")]
    ConnectionClosed,

    /// An I/O error occurred while reading from the socket.
    ///
    /// Typically caused by a low-level network failure or an unexpected
    /// socket interruption.
    #[error("I/O error while reading packet: {0}")]
    Io(#[from] std::io::Error),

    /// Not enough bytes are available in the buffer to read the packet length prefix.
    ///
    /// The prefix is a 2-byte (`u16`) field defining the body length of the packet.
    #[error("not enough bytes to read packet length prefix (need {required}, got {available})")]
    IncompletePrefix {
        /// Number of bytes currently in the buffer.
        available: usize,
        /// Number of bytes required to read the prefix.
        required: usize,
    },

    /// Not enough bytes in the buffer to read the declared full packet.
    ///
    /// Indicates that the packet was truncated or the connection was interrupted.
    #[error("packet not fully received (need {required}, got {available})")]
    IncompletePacket {
        /// Total bytes required for the full packet.
        required: usize,
        /// Bytes currently available in the buffer.
        available: usize,
    },

    /// The packet body is smaller than required for checksum or ID fields.
    ///
    /// The minimum body size includes at least the checksum (4 bytes) and the
    /// packet kind identifier (1 byte).
    #[error("packet too short: {actual} bytes available, expected at least {min} bytes")]
    TooShort {
        /// Number of bytes currently available.
        actual: usize,
        /// Minimum bytes required.
        min: usize,
    },

    /// The declared body length in the packet header is zero.
    #[error("packet body length declared as zero")]
    EmptyLength,

    /// The declared packet length exceeds the configured maximum allowed size.
    ///
    /// This prevents oversized or malicious packets from being processed.
    #[error("declared packet length ({declared} bytes) exceeds the maximum allowed ({max} bytes)")]
    LengthOutOfBounds {
        /// Declared total packet length.
        declared: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// The packet checksum does not match the computed value.
    ///
    /// Indicates that the packet payload was corrupted or tampered with.
    #[error("checksum mismatch: expected {expected:#010x}, actual {actual:#010x}")]
    ChecksumMismatch {
        /// Expected checksum value read from the packet.
        expected: u32,
        /// Actual computed checksum.
        actual: u32,
    },

    /// The packet ID read from the payload is invalid or unknown.
    ///
    /// The packet ID determines which packet type should be processed.
    #[error("unknown packet ID: {0:#04x}")]
    UnknownId(u8),
}

/// Asynchronous trait for reading and decoding login packets from a stream.
///
/// This trait provides an extension method for any type implementing
/// [`AsyncRead`], allowing it to read, accumulate, and decode complete
/// login packets following the protocol format.
pub(crate) trait LoginReadPacket {
    /// Reads and decodes a single login packet from the client stream.
    fn read_login_packet(
        &mut self,
        max_length: usize,
    ) -> impl Future<Output = Result<IncomingPacket, PacketReadError>>;
}

impl<T> LoginReadPacket for T
where
    T: AsyncRead + Unpin + Send + Sync,
{
    async fn read_login_packet(
        &mut self,
        max_length: usize,
    ) -> Result<IncomingPacket, PacketReadError> {
        trace!("Starting to read login packet from client stream");

        // Initialize a buffer to accumulate incoming bytes.
        let mut buffer = PacketBuffer::with_capacity(max_length);

        // Perform the socket read operation.
        let n = self.read(buffer.payload_mut()).await.map_err(|err| {
            warn!("I/O error while reading from socket: {:?}", err);
            PacketReadError::Io(err)
        })?;

        trace!("Read {} bytes from socket", n);

        // If zero bytes read, the connection was closed
        if n == 0 {
            warn!("Connection closed before packet was fully received");
            return Err(PacketReadError::ConnectionClosed);
        }

        // Limit the buffer size to the number of bytes actually read.
        buffer.truncate(n);

        let len = buffer.payload_len();
        trace!("Buffer now contains {} bytes", len);

        // Attempt to extract and parse a complete packet.
        match buffer.take_packet(max_length) {
            Ok(packet) => {
                debug!(
                    "Successfully parsed login packet ({} bytes total)",
                    packet.buffer.len()
                );
                Ok(packet)
            }
            Err(err) => {
                warn!("Failed to decode login packet: {}", err);
                Err(err)
            }
        }
    }
}
