//! Client look-at packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookAt {
    pub position: Position,
    pub floor: Floor,
    pub stack_position: u8,
}

impl Decodable for LookAt {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
            LookAt::decode(PacketKind::LookAt, &mut payload)
                .unwrap()
                .stack_position,
            3
        );
    }
}
