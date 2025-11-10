use bytes::Bytes;

use crate::packets::PACKET_KIND_SIZE;

mod keep_alive;
mod ping_latency;

pub mod prelude {
    pub use super::{
        Encodable, PacketKind, keep_alive::KeepAlivePacket, ping_latency::PingLatencyPacket,
    };
}

/// Represents a packet type that can be serialized into a binary format.
///
/// This trait defines how a packet is transformed into raw bytes for transmission
/// or storage. It provides a default encoding implementation that returns `None`,
/// which can be overridden in concrete packet types. Typically, packets will also
/// include a kind identifier via [`PacketKind`] when transmitted.
///
/// # Associated Constant
/// - [`KIND`]: The unique [`PacketKind`] that identifies this packet type.
///
/// # Methods
/// - [`encode`]: Encodes the packet’s payload into a `Bytes` buffer.
///
/// # Example
/// ```ignore
/// struct LoginPacket {
///     username: String,
/// }
///
/// impl Encodable for LoginPacket {
///     const KIND: PacketKind = PacketKind::Login;
///
///     fn encode(self) -> Option<Bytes> {
///         let mut encoder = Encoder::new();
///         encoder.put_string(&self.username);
///         Some(encoder.end())
///     }
/// }
///
/// // Produces a byte buffer for transmission
/// let packet = LoginPacket { username: "Alice".into() };
/// let encoded = packet.encode();
/// ```
///
/// This trait is typically paired with a corresponding `Decodable` trait to
/// reconstruct the packet from a received byte stream.
pub trait Encodable: Sized {
    /// Unique kind identifier for this packet type.
    const KIND: PacketKind;

    /// Encodes the packet payload into a binary representation.
    ///
    /// This method can be overridden to provide custom encoding logic.
    /// By default, it returns `None`, representing a packet with no payload.
    fn encode(self) -> Option<Bytes> {
        // No payload by default; only the packet kind is used
        None
    }

    fn encode_with_kind(self) -> Bytes {
        use crate::packets::encoder::Encoder;

        if let Some(bytes) = self.encode() {
            Encoder::with_capacity(PACKET_KIND_SIZE + bytes.len())
                .put_u8(Self::KIND as u8)
                .put_bytes(bytes)
                .end()
        } else {
            Encoder::with_capacity(PACKET_KIND_SIZE)
                .put_u8(Self::KIND as u8)
                .end()
        }
    }
}

/// Defines the possible kinds or categories of network packets.
///
/// Each [`PacketKind`] corresponds to a specific packet type that implements
/// the [`Encodable`] trait. This allows the system to determine how to serialize
/// and distinguish different packet variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketKind {
    /// Keeps the connection alive.
    KeepAlive = 29,
    /// Sent to measure latency between client and server.
    PingLatency = 30,
}

impl std::fmt::Display for PacketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAYLOAD: &[u8] = &[1, 2, 3, 4];

    struct Packet;

    impl Encodable for Packet {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn encode(self) -> Option<Bytes> {
            Some(Bytes::from_static(PAYLOAD))
        }
    }

    #[test]
    fn encode_packet_returns_expected_bytes() {
        let packet = Packet;
        let encoded = packet
            .encode()
            .expect("Encoding should produce a byte buffer");

        assert_eq!(
            encoded.as_ref(),
            PAYLOAD,
            "Encoded bytes should match the predefined payload"
        );
    }
}
