//! Previous floor components.

use bevy::prelude::*;
use std::{cmp::*, hash::*};

#[derive(Component, Hash, PartialEq, Eq, PartialOrd, Ord, Deref, Clone, Copy, Debug)]
#[component(immutable)]
/// Snapshot of the previous vertical layer coordinate.
pub struct PreviousFloor {
    /// Previous floor index in the world.
    pub z: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn should_order_previous_floors_by_z() {
        let mut floors = vec![PreviousFloor { z: 9 }, PreviousFloor { z: 0 }, PreviousFloor { z: 4 }];

        floors.sort();

        assert_eq!(
            floors,
            vec![
                PreviousFloor { z: 0 },
                PreviousFloor { z: 4 },
                PreviousFloor { z: 9 },
            ],
            "Derived ordering should sort previous floors by z"
        );
    }

    #[test]
    fn should_deref_to_previous_floor_value() {
        let floor = PreviousFloor { z: 12 };

        assert_eq!(
            *floor, 12,
            "Deref should expose the inner previous floor value for ergonomic reads"
        );
    }

    #[test]
    fn should_hash_and_compare_equal_previous_floors_consistently() {
        let mut hashed = HashSet::new();
        let mut ordered = BTreeSet::new();

        hashed.insert(PreviousFloor { z: 2 });
        hashed.insert(PreviousFloor { z: 2 });
        ordered.insert(PreviousFloor { z: 2 });
        ordered.insert(PreviousFloor { z: 2 });

        assert_eq!(
            hashed.len(),
            1,
            "Equal previous floors should hash to one set entry"
        );

        assert_eq!(
            ordered.len(),
            1,
            "Equal previous floors should occupy one ordered set entry"
        );
    }
}
