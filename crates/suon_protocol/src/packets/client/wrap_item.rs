//! Client wrap-item packet.

use suon_position::{floor::Floor, position::Position};

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to wrap the referenced item at its current slot.
///
/// The wire payload names the exact position and stack slot of the item to be
/// transformed into its wrapped representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrapItem {
    /// Map coordinates of the tile or container slot holding the item to wrap.
    pub position: Position,
    /// Floor component of the addressed coordinates.
    pub floor: Floor,
    /// Advertised item type currently present at the addressed slot.
    pub item_id: u16,
    /// Stack slot of the item that should be transformed into its wrapped form.
    pub stack_position: u8,
}

impl Decodable for WrapItem {
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
    fn should_decode_wrap_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3];

        assert_eq!(
            WrapItem::decode(PacketKind::WrapItem, &mut payload)
                .unwrap()
                .item_id,
            0x1234
        );
    }
}
