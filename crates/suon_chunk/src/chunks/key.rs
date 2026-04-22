//! Internal chunk-key representation.
//!
//! This module converts world-space positions into stable keys used by the
//! [`crate::chunks::Chunks`] registry.

use crate::CHUNK_EXP;
use suon_position::prelude::*;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
/// Compact key identifying the chunk that contains a world position.
pub(crate) struct ChunkKey(u32);

impl ChunkKey {
    /// Builds a chunk key from world-space coordinates.
    ///
    /// Coordinates are shifted by [`crate::CHUNK_EXP`] to collapse every tile in
    /// the same chunk footprint into one registry key.
    pub(crate) fn new(x: u16, y: u16) -> Self {
        let chunk_x = x as u32 >> CHUNK_EXP;
        let chunk_y = y as u32 >> CHUNK_EXP;
        Self((chunk_x << 16) | chunk_y)
    }
}

impl From<Position> for ChunkKey {
    fn from(pos: Position) -> Self {
        Self::new(pos.x, pos.y)
    }
}

impl From<&Position> for ChunkKey {
    fn from(pos: &Position) -> Self {
        Self::new(pos.x, pos.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CHUNK_SIZE;

    #[test]
    fn should_generate_same_key_for_positions_inside_same_chunk() {
        const FIRST_POSITION: Position = Position { x: 0, y: 0 };
        const SECOND_POSITION: Position = Position {
            x: CHUNK_SIZE as u16 - 1,
            y: CHUNK_SIZE as u16 - 1,
        };

        // Any position inside the same chunk footprint must resolve to the same key.
        assert_eq!(
            ChunkKey::from(FIRST_POSITION),
            ChunkKey::from(SECOND_POSITION),
            "Positions inside the same chunk bounds should share one key"
        );
    }

    #[test]
    fn should_generate_different_key_when_crossing_chunk_boundary() {
        const ORIGIN_POSITION: Position = Position { x: 0, y: 0 };
        const X_BOUNDARY_POSITION: Position = Position {
            x: CHUNK_SIZE as u16,
            y: 0,
        };
        const Y_BOUNDARY_POSITION: Position = Position {
            x: 0,
            y: CHUNK_SIZE as u16,
        };

        // Crossing a chunk boundary on either axis must produce a different key.
        assert_ne!(
            ChunkKey::from(ORIGIN_POSITION),
            ChunkKey::from(X_BOUNDARY_POSITION),
            "Crossing the x chunk boundary should produce a different key"
        );

        assert_ne!(
            ChunkKey::from(ORIGIN_POSITION),
            ChunkKey::from(Y_BOUNDARY_POSITION),
            "Crossing the y chunk boundary should produce a different key"
        );
    }

    #[test]
    fn should_match_owned_and_borrowed_position_conversions() {
        const POSITION: Position = Position { x: 123, y: 456 };

        // Borrowed and owned conversions should be interchangeable for callers.
        assert_eq!(
            ChunkKey::from(POSITION),
            ChunkKey::from(&POSITION),
            "Owned and borrowed conversions should be identical"
        );
    }
}
