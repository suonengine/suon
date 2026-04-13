//! Client use-item-with-creature packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to apply an item directly to a creature target.
///
/// The item source is encoded exactly like a regular use action, followed by
/// the target creature id that should receive the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UseItemWithCreature {
    /// Map coordinates of the tile or container slot providing the item.
    pub position: Position,
    /// Floor component of the source coordinates.
    pub floor: Floor,
    /// Advertised item type currently present at the addressed slot.
    pub item_id: u16,
    /// Stack slot of the item inside the addressed tile or container.
    pub stack_position: u8,
    /// Creature identifier that should receive the item effect.
    pub creature_id: u32,
}

impl Decodable for UseItemWithCreature {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
            floor: bytes.get_floor()?,
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            creature_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_use_item_with_creature() {
        let mut payload: &[u8] = &[1, 0, 2, 0, 7, 0x34, 0x12, 3, 0x78, 0x56, 0x34, 0x12];
        let packet =
            UseItemWithCreature::decode(PacketKind::UseItemWithCreature, &mut payload).unwrap();
        assert_eq!(packet.creature_id, 0x12345678);
    }
}
