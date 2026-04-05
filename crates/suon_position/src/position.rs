//! Current world-space position components.

use bevy::prelude::*;
use std::{cmp::*, hash::*};

#[derive(Component, Clone, Copy, Debug)]
#[component(immutable)]
/// Immutable tile coordinate in world space.
///
/// # Examples
/// ```
/// use suon_position::position::Position;
///
/// let position = Position { x: 9, y: 3 };
///
/// assert_eq!(position.x, 9);
/// assert_eq!(position.y, 3);
/// ```
pub struct Position {
    /// Horizontal world coordinate.
    pub x: u16,
    /// Vertical world coordinate.
    pub y: u16,
}

impl PartialEq<Position> for Position {
    fn eq(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for Position {}

impl Hash for Position {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.x);
        state.write_u16(self.y);
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y.cmp(&other.y).then_with(|| self.x.cmp(&other.x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn should_compare_positions_by_y_then_x() {
        let mut positions = vec![
            Position { x: 4, y: 2 },
            Position { x: 1, y: 1 },
            Position { x: 3, y: 1 },
            Position { x: 2, y: 0 },
        ];

        positions.sort();

        assert_eq!(
            positions,
            vec![
                Position { x: 2, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 3, y: 1 },
                Position { x: 4, y: 2 },
            ],
            "Position ordering should sort by y first and x second"
        );
    }

    #[test]
    fn should_treat_matching_coordinates_as_equal_and_hash_to_same_set_entry() {
        let first = Position { x: 8, y: 13 };
        let second = Position { x: 8, y: 13 };
        let mut set = HashSet::new();

        assert_eq!(
            first, second,
            "Positions with identical coordinates should compare equal"
        );

        set.insert(first);
        set.insert(second);

        assert_eq!(
            set.len(),
            1,
            "Hashing should collapse identical coordinates to one set entry"
        );
    }

    #[test]
    fn should_work_as_btree_key_using_total_order() {
        let mut set = BTreeSet::new();
        set.insert(Position { x: 9, y: 3 });
        set.insert(Position { x: 1, y: 3 });
        set.insert(Position { x: 5, y: 2 });

        let ordered: Vec<_> = set.into_iter().collect();

        assert_eq!(
            ordered,
            vec![
                Position { x: 5, y: 2 },
                Position { x: 1, y: 3 },
                Position { x: 9, y: 3 },
            ],
            "The Ord implementation should be stable enough for ordered map/set usage"
        );
    }

    #[test]
    fn should_clone_and_copy_positions_without_changing_coordinates() {
        let original = Position { x: 77, y: 88 };
        let copied = original;
        let cloned = original.clone();

        assert_eq!(
            copied, original,
            "Copy should preserve the original coordinates exactly"
        );
        assert_eq!(
            cloned, original,
            "Clone should preserve the original coordinates exactly"
        );
    }
}
