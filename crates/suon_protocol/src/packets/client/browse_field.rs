//! Client browse-field packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseFieldPacket {
    pub position: Position,
    pub floor: Floor,
}

impl Decodable for BrowseFieldPacket {
    const KIND: PacketKind = PacketKind::BrowseField;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_browse_field() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7];
        assert_eq!(BrowseFieldPacket::decode(&mut payload).unwrap().floor.z, 7);
    }
}
