//! Client change-podium packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to start podium-appearance editing for a decorative item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangePodium {
    /// Map coordinates of the podium item to edit.
    pub position: Position,

    /// Floor component of the podium item coordinates.
    pub floor: Floor,

    /// Advertised item type currently present at the addressed podium slot.
    pub item_id: u16,

    /// Stack slot of the podium item inside the addressed tile.
    pub stack_position: u8,
}

impl Decodable for ChangePodium {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_change_podium() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];
        let packet = ChangePodium::decode(PacketKind::ChangePodium, &mut payload).unwrap();
        assert_eq!(packet.item_id, 0x1234);
    }
}
