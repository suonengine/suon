//! Current floor components.

use bevy::prelude::*;
use std::{cmp::*, hash::*};

#[derive(Component, Hash, PartialEq, Eq, PartialOrd, Ord, Deref, Clone, Copy, Debug)]
#[component(immutable)]
/// Immutable vertical layer coordinate.
///
/// # Examples
/// ```
/// use suon_position::floor::Floor;
///
/// let floor = Floor { z: 4 };
///
/// assert_eq!(*floor, 4);
/// assert_eq!(floor.z, 4);
/// ```
pub struct Floor {
    /// Floor index in the world.
    pub z: u8,
}

impl From<u8> for Floor {
    fn from(z: u8) -> Self {
        Self { z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn should_order_floors_by_z() {
        let mut floors = vec![Floor { z: 3 }, Floor { z: 1 }, Floor { z: 2 }];

        floors.sort();

        assert_eq!(
            floors,
            vec![Floor { z: 1 }, Floor { z: 2 }, Floor { z: 3 }],
            "Derived ordering should sort floors by z"
        );
    }

    #[test]
    fn should_deref_to_floor_value() {
        let floor = Floor { z: 7 };

        assert_eq!(
            *floor, 7,
            "Deref should expose the inner floor value for ergonomic reads"
        );
    }

    #[test]
    fn should_hash_and_compare_equal_floors_consistently() {
        let mut hashed = HashSet::new();
        let mut ordered = BTreeSet::new();

        hashed.insert(Floor { z: 4 });
        hashed.insert(Floor { z: 4 });
        ordered.insert(Floor { z: 4 });
        ordered.insert(Floor { z: 4 });

        assert_eq!(hashed.len(), 1, "Equal floors should hash to one set entry");

        assert_eq!(
            ordered.len(),
            1,
            "Equal floors should occupy one ordered set entry"
        );
    }

    #[test]
    fn should_clone_and_copy_floors_without_changing_z() {
        fn clone_value<T: Clone>(value: &T) -> T {
            value.clone()
        }

        let original = Floor { z: 9 };
        let copied = original;
        let cloned = clone_value(&original);

        assert_eq!(copied, original, "Copy should preserve the floor value");
        assert_eq!(cloned, original, "Clone should preserve the floor value");
    }

    #[test]
    fn should_create_floor_from_u8() {
        assert_eq!(Floor::from(7), Floor { z: 7 });
    }
}
