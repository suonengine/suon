//! Client trail packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrailPacket {
    pub creature_id: u32,
}

impl Decodable for TrailPacket {
    const KIND: PacketKind = PacketKind::Trail;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            creature_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_trail() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
        assert_eq!(
            TrailPacket::decode(&mut payload).unwrap().creature_id,
            0x12345678
        );
    }
}
