use bytes::Bytes;
use std::time::Instant;
use suon_checksum::Adler32Checksum;
use suon_protocol::packets::client::PacketKind;

pub mod login;
pub mod server_name;
pub mod subsequent;

/// Represents a decoded packet received from a client.
#[derive(Debug)]
pub(crate) struct IncomingPacket {
    /// Timestamp of when the packet was decoded.
    pub timestamp: Instant,

    /// Checksum declared in the packet header.
    pub checksum: Option<Adler32Checksum>,

    /// Packet kind.
    pub kind: PacketKind,

    /// Raw packet payload.
    pub buffer: Bytes,
}
