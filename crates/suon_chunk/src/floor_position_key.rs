//! Compact keys for floor-position lookup tables.

use suon_position::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Packed representation of a floor-position pair.
///
/// [`Floor`] is an `u8` and [`Position`] stores two `u16` coordinates, so the
/// full spatial key fits in 40 bits. Keeping it as one integer makes occupancy
/// and navigation maps hash a single scalar instead of nested or compound
/// component values.
pub(crate) struct FloorPositionKey(u64);

impl FloorPositionKey {
    /// Builds a compact key from a floor-position pair.
    pub(crate) const fn new(floor: Floor, position: Position) -> Self {
        Self(((floor.z as u64) << 32) | ((position.x as u64) << 16) | position.y as u64)
    }
}
