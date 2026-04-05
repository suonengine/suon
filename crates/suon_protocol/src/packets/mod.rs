//! Packet primitives, codecs, and typed packet families.

pub mod client;
pub mod decoder;
pub mod encoder;
pub mod server;

/// Number of bytes used by the packet KIND field.
///
/// # Examples
/// ```
/// use suon_protocol::packets::PACKET_KIND_SIZE;
///
/// assert_eq!(PACKET_KIND_SIZE, 1);
/// ```
pub const PACKET_KIND_SIZE: usize = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_expose_expected_packet_kind_size() {
        assert_eq!(
            PACKET_KIND_SIZE, 1,
            "Packet kind encoding should continue to use exactly one byte"
        );
    }
}
