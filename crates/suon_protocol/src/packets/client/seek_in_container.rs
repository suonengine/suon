//! Client seek-in-container packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to seek to an index inside an open container.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::SeekInContainer};
///
/// let mut payload: &[u8] = &[3, 0x34, 0x12, 7];
/// let packet = SeekInContainer::decode(PacketKind::SeekInContainer, &mut payload).unwrap();
///
/// assert_eq!(packet.container_id, 3);
/// assert_eq!(packet.index, 0x1234);
/// assert_eq!(packet.primary_type, 7);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeekInContainer {
    /// Container identifier being paged.
    pub container_id: u8,

    /// Item index requested inside the container.
    pub index: u16,

    /// Primary type filter forwarded by the client.
    pub primary_type: u8,
}

impl Decodable for SeekInContainer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            container_id: bytes.get_u8()?,
            index: bytes.get_u16()?,
            primary_type: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_seek_in_container() {
        let mut payload: &[u8] = &[3, 0x34, 0x12, 7];

        let packet = SeekInContainer::decode(PacketKind::SeekInContainer, &mut payload)
            .expect("SeekInContainer packets should decode the container, index, and type");

        assert_eq!(packet.container_id, 3);
        assert_eq!(packet.index, 0x1234);
        assert_eq!(packet.primary_type, 7);
        assert!(payload.is_empty());
    }
}
