use thiserror::Error;

mod keep_alive;
mod ping_latency;

pub mod prelude {
    pub use super::{
        Decodable, DecodableError, PacketKind, keep_alive::KeepAlivePacket,
        ping_latency::PingLatencyPacket,
    };
}

/// Errors that can occur while decoding a packet.
#[derive(Debug, Error)]
pub enum DecodableError {
    /// Wraps a lower-level decoding error.
    #[error("failed to decode packet: {0}")]
    Decoder(#[from] crate::packets::decoder::DecoderError),
}

/// Represents a packet that can be decoded from a binary buffer.
///
/// This trait defines how a packet is reconstructed from raw bytes received
/// over a network or read from storage. Each packet type has a unique [`PacketKind`]
/// that identifies it and allows the system to dispatch the correct decoding logic.
///
/// # Associated Constant
/// - [`Self::KIND`]: The unique [`PacketKind`] that identifies this packet type.
///
/// # Methods
/// - [`Self::decode`]: Decodes the packet instance from a raw byte slice.
///
/// # Example
/// ```
/// use suon_protocol::packets::client::{Decodable, DecodableError, PacketKind};
///
/// struct LoginPacket {
///     username: String,
/// }
///
/// impl Decodable for LoginPacket {
///     const KIND: PacketKind = PacketKind::Login;
///
///     fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
///         use suon_protocol::packets::decoder::Decoder;
///
///         let username = (&mut *bytes).get_string()?;
///         Ok(LoginPacket { username })
///     }
/// }
///
/// let mut buffer: &[u8] = &[5, 0, b'A', b'l', b'i', b'c', b'e'];
/// let packet = LoginPacket::decode(&mut buffer).unwrap();
///
/// assert_eq!(packet.username, "Alice");
/// ```
///
/// This trait is typically paired with the server-side
/// [`crate::packets::server::Encodable`] trait to allow symmetric serialization
/// and deserialization of packet types.
pub trait Decodable: Sized {
    /// Unique kind identifier for this packet type.
    const KIND: PacketKind;

    /// Decodes the packet instance from a raw byte slice.
    ///
    /// Implementers should read the buffer according to the expected packet structure.
    /// Returns an error if the buffer is incomplete or contains invalid data.
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError>;
}

/// Defines the possible kinds or categories of network packets.
///
/// Each [`PacketKind`] corresponds to a specific packet type that implements
/// the [`Decodable`] trait. This allows the system to determine how to
/// deserialize and distinguish different packet variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketKind {
    /// Internal packet sent by the client as the **first message**.
    ServerName = 0,

    /// Sent when a client attempts to log in.
    Login = 10,
    /// Sent when a client logs out.
    Logout = 20,

    /// Sent to measure latency between client and server.
    PingLatency = 29,
    /// Keeps the connection alive.
    KeepAlive = 30,
}

impl TryFrom<u8> for PacketKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ServerName),
            10 => Ok(Self::Login),
            20 => Ok(Self::Logout),
            29 => Ok(Self::PingLatency),
            30 => Ok(Self::KeepAlive),
            _ => Err(value),
        }
    }
}

impl std::fmt::Display for PacketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (0x{:02X})", self, *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Packet;

    impl Decodable for Packet {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
            if bytes.is_empty() {
                Err(DecodableError::Decoder(
                    crate::packets::decoder::DecoderError::Incomplete {
                        expected: 1,
                        available: 0,
                    },
                ))
            } else {
                Ok(Packet)
            }
        }
    }

    #[test]
    fn decode_packet_returns_error_on_empty_buffer() {
        let mut buffer: &[u8] = &[];

        let error = Packet::decode(&mut buffer)
            .expect_err("Expected DecoderError::Incomplete for empty buffer");

        match error {
            DecodableError::Decoder(crate::packets::decoder::DecoderError::Incomplete {
                expected,
                available,
            }) => {
                assert!(
                    expected == 1,
                    "Expected 1 byte to be required, got {}",
                    expected
                );
                assert!(
                    available == 0,
                    "Expected 0 bytes available, got {}",
                    available
                );
            }
            other => {
                panic!("Unexpected error variant: {:?}", other);
            }
        }
    }

    #[test]
    fn decode_packet_succeeds_with_non_empty_buffer() {
        const PAYLOAD: &[u8] = &[42];

        let mut buffer: &[u8] = PAYLOAD;

        let packet_result = Packet::decode(&mut buffer);
        assert!(
            packet_result.is_ok(),
            "Decoding should succeed with non-empty buffer"
        );

        let packet = packet_result.unwrap();
        assert!(
            matches!(packet, Packet),
            "Decoded packet should be of type Packet"
        );
    }

    #[test]
    fn packet_kind_should_convert_from_wire_values_and_format_for_logs() {
        assert_eq!(
            PacketKind::try_from(30),
            Ok(PacketKind::KeepAlive),
            "Wire value 30 should decode to the keep-alive client packet kind"
        );
        assert_eq!(
            PacketKind::PingLatency.to_string(),
            "PingLatency (0x1D)",
            "Display should include both the variant name and hexadecimal id"
        );
    }
}
