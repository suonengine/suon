//! Previous world-space position components.

use bevy::prelude::*;
use std::{cmp::*, hash::*};

#[derive(Component, Clone, Copy, Debug)]
#[component(immutable)]
/// Snapshot of the previous tile coordinate in world space.
///
/// # Examples
/// ```
/// use suon_position::prelude::*;
///
/// let previous = PreviousPosition { x: 5, y: 6 };
///
/// assert_eq!(previous.x, 5);
/// assert_eq!(previous.y, 6);
/// ```
pub struct PreviousPosition {
    /// Previous horizontal world coordinate.
    pub x: u16,

    /// Previous vertical world coordinate.
    pub y: u16,
}

impl PartialEq<PreviousPosition> for PreviousPosition {
    fn eq(&self, other: &PreviousPosition) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for PreviousPosition {}

impl Hash for PreviousPosition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u16(self.x);
        state.write_u16(self.y);
    }
}

impl PartialOrd for PreviousPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PreviousPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y.cmp(&other.y).then_with(|| self.x.cmp(&other.x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn should_compare_previous_positions_by_y_then_x() {
        let mut positions = vec![
            PreviousPosition { x: 2, y: 4 },
            PreviousPosition { x: 3, y: 1 },
            PreviousPosition { x: 1, y: 1 },
        ];

        positions.sort();

        assert_eq!(
            positions,
            vec![
                PreviousPosition { x: 1, y: 1 },
                PreviousPosition { x: 3, y: 1 },
                PreviousPosition { x: 2, y: 4 },
            ],
            "PreviousPosition ordering should sort by y first and x second"
        );
    }

    #[test]
    fn should_treat_matching_previous_coordinates_as_equal_and_hash_to_same_entry() {
        let first = PreviousPosition { x: 6, y: 9 };
        let second = PreviousPosition { x: 6, y: 9 };
        let mut set = HashSet::new();

        set.insert(first);
        set.insert(second);

        assert_eq!(
            first, second,
            "Previous positions with identical coordinates should compare equal"
        );

        assert_eq!(
            set.len(),
            1,
            "Hashing should collapse identical previous coordinates to one set entry"
        );
    }

    #[test]
    fn should_work_as_ordered_set_key() {
        let mut set = BTreeSet::new();
        set.insert(PreviousPosition { x: 8, y: 8 });
        set.insert(PreviousPosition { x: 1, y: 7 });
        set.insert(PreviousPosition { x: 4, y: 8 });

        let ordered: Vec<_> = set.into_iter().collect();

        assert_eq!(
            ordered,
            vec![
                PreviousPosition { x: 1, y: 7 },
                PreviousPosition { x: 4, y: 8 },
                PreviousPosition { x: 8, y: 8 },
            ],
            "The total order should be stable enough for ordered collections"
        );
    }

    #[test]
    fn should_clone_and_copy_previous_positions_without_changing_coordinates() {
        fn clone_value<T: Clone>(value: &T) -> T {
            value.clone()
        }

        let original = PreviousPosition { x: 44, y: 55 };
        let copied = original;
        let cloned = clone_value(&original);

        assert_eq!(
            copied, original,
            "Copy should preserve the previous position coordinates"
        );

        assert_eq!(
            cloned, original,
            "Clone should preserve the previous position coordinates"
        );
    }
}
