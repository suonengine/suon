pub mod client;
pub mod decoder;
pub mod encoder;
pub mod server;

/// Number of bytes used by the packet KIND field.
pub const PACKET_KIND_SIZE: usize = 1;
