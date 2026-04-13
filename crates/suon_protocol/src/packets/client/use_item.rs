//! Client use-item packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to activate an item directly from a tile slot.
///
/// The payload points to the item instance on the map or container and carries
/// a use index byte for protocol variants that multiplex multiple actions from
/// the same item reference.
///
/// # Examples
///
/// ```rust
/// use suon_protocol::packets::client::prelude::{Decodable, PacketKind, UseItem};
///
/// let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 4];
/// let packet = UseItem::decode(PacketKind::UseItem, &mut payload).unwrap();
///
/// assert_eq!(packet.item_id, 0x1234);
/// assert_eq!(packet.use_index, 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseItem {
    /// Map coordinates of the tile or container slot providing the item.
    pub position: Position,
    /// Floor component of the source coordinates.
    pub floor: Floor,
    /// Advertised item type currently present at the addressed slot.
    pub item_id: u16,
    /// Stack slot of the item inside the addressed tile or container.
    pub stack_position: u8,
    /// Protocol subaction byte selecting which use behavior should run.
    pub use_index: u8,
}

impl Decodable for UseItem {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            use_index: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_use_item() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 4];
        let packet = UseItem::decode(PacketKind::UseItem, &mut payload).unwrap();
        assert_eq!(packet.use_index, 4);
    }
}
