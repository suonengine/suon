//! Client seek-in-container packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeekInContainerPacket {
    pub container_id: u8,
    pub index: u16,
}

impl Decodable for SeekInContainerPacket {
    const KIND: PacketKind = PacketKind::SeekInContainer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            container_id: bytes.get_u8()?,
            index: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_seek_in_container() {
        let mut payload: &[u8] = &[3, 0x34, 0x12];
        let packet = SeekInContainerPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.index, 0x1234);
    }
}
