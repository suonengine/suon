//! Client target packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetPacket {
    pub creature_id: u32,
}

impl Decodable for TargetPacket {
    const KIND: PacketKind = PacketKind::Target;

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
    fn should_decode_target() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
        assert_eq!(
            TargetPacket::decode(&mut payload).unwrap().creature_id,
            0x12345678
        );
    }
}
