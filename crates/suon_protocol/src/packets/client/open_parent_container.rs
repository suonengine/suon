//! Client open-parent-container packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Packet sent by the client to open the parent container from a depot-search result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenParentContainerPacket {
    /// Position of the container to open.
    pub position: Position,
}

impl Decodable for OpenParentContainerPacket {
    const KIND: PacketKind = PacketKind::OpenParentContainer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_parent_container() {
        let mut payload: &[u8] = &[0x34, 0x12, 0x78, 0x56];
        let packet = OpenParentContainerPacket::decode(&mut payload)
            .expect("OpenParentContainer packets should decode a position");
        assert_eq!(packet.position.x, 0x1234);
        assert_eq!(packet.position.y, 0x5678);
        assert!(payload.is_empty());
    }
}
