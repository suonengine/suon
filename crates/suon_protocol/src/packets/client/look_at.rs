//! Client look-at packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookAtPacket {
    pub position: Position,
    pub floor: Floor,
    pub stack_position: u8,
}

impl Decodable for LookAtPacket {
    const KIND: PacketKind = PacketKind::LookAt;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            stack_position: {
                let _item_id = bytes.get_u16()?;
                bytes.get_u8()?
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_look_at() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];
        assert_eq!(
            LookAtPacket::decode(&mut payload).unwrap().stack_position,
            3
        );
    }
}
