//! Client browse-field packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;
use suon_position::{floor::Floor, position::Position};

/// Packet sent by the client to request the contents of a specific map tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// Map coordinates of the tile whose contents should be browsed.
    pub position: Position,

    /// Floor component of the requested tile coordinates.
    pub floor: Floor,
}

impl Decodable for Tile {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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
        let packet = Tile::decode(PacketKind::BrowseField, &mut payload)
            .expect("BrowseField packets should decode a full tile position");

        assert_eq!(packet.position, Position { x: 1, y: 2 });
        assert_eq!(packet.floor.z, 7);
        assert!(payload.is_empty());
    }
}
