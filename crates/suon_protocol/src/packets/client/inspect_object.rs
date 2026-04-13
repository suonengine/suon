//! Client inspect-object packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Object-inspection target requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectObjectKind {
    /// Inspects an object on the map.
    MapObject {
        /// Position of the object being inspected.
        position: Position,
    },
    /// Inspects an item listed in the NPC trade window.
    NpcTradeItem {
        /// Item id being inspected.
        item_id: u16,
        /// Item count shown to the user.
        count: u8,
    },
    /// Inspects an item shown in the cyclopedia.
    CyclopediaItem {
        /// Item id being inspected.
        item_id: u16,
        /// Item count shown to the user.
        count: u8,
    },
}

/// Packet sent by the client to inspect an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectObject {
    /// Inspection target requested by the client.
    pub target: InspectObjectKind,
}

impl Decodable for InspectObject {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let target = match bytes.get_u8()? {
            0 => InspectObjectKind::MapObject {
                position: bytes.get_position()?,
            },
            1 => InspectObjectKind::NpcTradeItem {
                item_id: bytes.get_u16()?,
                count: bytes.get_u8()?,
            },
            3 => InspectObjectKind::CyclopediaItem {
                item_id: bytes.get_u16()?,
                count: bytes.get_u8()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "inspection_type",
                    value,
                });
            }
        };

        Ok(Self { target })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_map_object_inspection() {
        let mut payload: &[u8] = &[0, 0x34, 0x12, 0x78, 0x56];

        let packet = InspectObject::decode(PacketKind::InspectObject, &mut payload)
            .expect("InspectObject packets should decode map-object inspections");

        assert_eq!(
            packet.target,
            InspectObjectKind::MapObject {
                position: Position {
                    x: 0x1234,
                    y: 0x5678,
                },
            }
        );
    }

    #[test]
    fn should_decode_cyclopedia_item_inspection() {
        let mut payload: &[u8] = &[3, 0x78, 0x56, 9];

        let packet = InspectObject::decode(PacketKind::InspectObject, &mut payload)
            .expect("InspectObject packets should decode cyclopedia inspections");

        assert_eq!(
            packet.target,
            InspectObjectKind::CyclopediaItem {
                item_id: 0x5678,
                count: 9,
            }
        );
        assert!(payload.is_empty());
    }
}
