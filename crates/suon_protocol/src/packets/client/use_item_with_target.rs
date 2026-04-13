//! Client use-item-with-target packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to use one item on another map or container
/// target.
///
/// The payload fully identifies both the source item reference and the target
/// item reference, allowing the server to resolve cross-item interactions such
/// as using tools on objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseItemWithTarget {
    /// Map coordinates of the source tile or container slot providing the item.
    pub from_position: Position,
    /// Floor component of the source coordinates.
    pub from_floor: Floor,
    /// Advertised item type currently present at the source slot.
    pub from_item_id: u16,
    /// Stack slot of the source item inside the addressed tile or container.
    pub from_stack_position: u8,
    /// Map coordinates of the destination tile or container slot.
    pub to_position: Position,
    /// Floor component of the destination coordinates.
    pub to_floor: Floor,
    /// Advertised item type currently present at the destination slot.
    pub to_item_id: u16,
    /// Stack slot of the destination item inside the addressed tile or container.
    pub to_stack_position: u8,
}

impl Decodable for UseItemWithTarget {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            from_position: bytes.get_position()?,
            from_floor: bytes.get_floor()?,
            from_item_id: bytes.get_u16()?,
            from_stack_position: bytes.get_u8()?,
            to_position: bytes.get_position()?,
            to_floor: bytes.get_floor()?,
            to_item_id: bytes.get_u16()?,
            to_stack_position: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_use_item_with_target() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 4, 0, 5, 0, 8, 0x78, 0x56, 9];
        let packet =
            UseItemWithTarget::decode(PacketKind::UseItemWithTarget, &mut payload).unwrap();
        assert_eq!(packet.to_item_id, 0x5678);
    }
}
