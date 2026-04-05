//! Step directions and position arithmetic helpers.

use std::{
    cmp::*,
    ops::{Add, *},
};
use suon_position::position::Position;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
/// One-tile direction used by step-based movement.
pub enum StepDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl StepDirection {
    /// Returns the `(dx, dy)` offset represented by this direction.
    ///
    /// # Examples
    /// ```
    /// use suon_movement::prelude::StepDirection;
    ///
    /// assert_eq!(StepDirection::North.offset(), (0, 1));
    /// assert_eq!(StepDirection::SouthWest.offset(), (-1, -1));
    /// ```
    pub fn offset(&self) -> (isize, isize) {
        match self {
            StepDirection::North => (0, 1),
            StepDirection::NorthEast => (1, 1),
            StepDirection::East => (1, 0),
            StepDirection::SouthEast => (1, -1),
            StepDirection::South => (0, -1),
            StepDirection::SouthWest => (-1, -1),
            StepDirection::West => (-1, 0),
            StepDirection::NorthWest => (-1, 1),
        }
    }
}

impl Add<StepDirection> for Position {
    type Output = Position;

    fn add(self, direction: StepDirection) -> Self::Output {
        let (dx, dy) = direction.offset();

        Position {
            x: match dx {
                1 => self.x.saturating_add(1),
                -1 => self.x.saturating_sub(1),
                _ => self.x,
            },
            y: match dy {
                1 => self.y.saturating_add(1),
                -1 => self.y.saturating_sub(1),
                _ => self.y,
            },
        }
    }
}

impl Sub<StepDirection> for Position {
    type Output = Position;

    fn sub(self, direction: StepDirection) -> Self::Output {
        let (dx, dy) = direction.offset();

        Position {
            x: match dx {
                1 => self.x.saturating_sub(1),
                -1 => self.x.saturating_add(1),
                _ => self.x,
            },
            y: match dy {
                1 => self.y.saturating_sub(1),
                -1 => self.y.saturating_add(1),
                _ => self.y,
            },
        }
    }
}

impl AddAssign<StepDirection> for Position {
    fn add_assign(&mut self, direction: StepDirection) {
        *self = *self + direction;
    }
}

impl SubAssign<StepDirection> for Position {
    fn sub_assign(&mut self, direction: StepDirection) {
        *self = *self - direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_compare_directions_for_equality_and_uniqueness() {
        const DIRECTION_NORTH: StepDirection = StepDirection::North;
        const DIRECTION_NORTH_WEST: StepDirection = StepDirection::NorthWest;

        assert_eq!(
            DIRECTION_NORTH,
            StepDirection::North,
            "The same direction variants must be equal to each other"
        );

        assert_ne!(
            DIRECTION_NORTH, DIRECTION_NORTH_WEST,
            "Different direction variants must not be equal"
        );
    }

    #[test]
    fn should_sort_directions_by_declaration_order() {
        let mut directions = vec![
            StepDirection::NorthWest,
            StepDirection::South,
            StepDirection::NorthEast,
            StepDirection::North,
        ];

        directions.sort();

        let expected_order = vec![
            StepDirection::North,
            StepDirection::NorthEast,
            StepDirection::South,
            StepDirection::NorthWest,
        ];

        assert_eq!(
            directions, expected_order,
            "The directions should be sorted according to their clockwise declaration order"
        );

        assert!(
            StepDirection::North < StepDirection::NorthEast,
            "North should be considered less than NorthEast based on declaration sequence"
        );

        assert!(
            StepDirection::SouthEast < StepDirection::South,
            "SouthEast should be considered less than South based on declaration sequence"
        );
    }

    #[test]
    fn should_copy_directions_by_value() {
        const ORIGINAL_DIRECTION: StepDirection = StepDirection::SouthWest;

        let copied_direction = ORIGINAL_DIRECTION;

        assert_eq!(
            copied_direction, ORIGINAL_DIRECTION,
            "The copied value must be identical to the original value"
        );
    }

    #[test]
    fn should_format_debug_output_consistently() {
        const DIRECTION: StepDirection = StepDirection::SouthEast;

        let formatted_string = format!("{:?}", DIRECTION);

        assert_eq!(
            formatted_string, "SouthEast",
            "The Debug format output must match the variant name exactly for logging purposes"
        );
    }

    #[test]
    fn should_keep_ordering_consistent_across_the_full_range() {
        const FIRST_VARIANT: StepDirection = StepDirection::North;
        const LAST_VARIANT: StepDirection = StepDirection::NorthWest;

        assert!(
            FIRST_VARIANT < LAST_VARIANT,
            "The first variant in the clockwise declaration must be less than the last variant"
        );

        assert!(
            StepDirection::East < StepDirection::West,
            "East (index 2) must be less than West (index 6) according to Ord implementation"
        );
    }

    #[test]
    fn should_expose_expected_coordinate_offsets() {
        const NORTH_OFFSET: (isize, isize) = (0, 1);
        const SOUTH_WEST_OFFSET: (isize, isize) = (-1, -1);
        const EAST_OFFSET: (isize, isize) = (1, 0);

        assert_eq!(
            StepDirection::North.offset(),
            NORTH_OFFSET,
            "North must move exactly one unit up on the Y axis"
        );

        assert_eq!(
            StepDirection::East.offset(),
            EAST_OFFSET,
            "East must move exactly one unit right on the X axis"
        );

        assert_eq!(
            StepDirection::South.offset(),
            (0, -1),
            "South must move exactly one unit down on the Y axis"
        );

        assert_eq!(
            StepDirection::West.offset(),
            (-1, 0),
            "West must move exactly one unit left on the X axis"
        );

        assert_eq!(
            StepDirection::NorthEast.offset(),
            (1, 1),
            "NorthEast must move positively on both X and Y axes"
        );

        assert_eq!(
            StepDirection::SouthEast.offset(),
            (1, -1),
            "SouthEast must move positively on X and negatively on Y"
        );

        assert_eq!(
            StepDirection::SouthWest.offset(),
            SOUTH_WEST_OFFSET,
            "SouthWest must move negatively on both X and Y axes"
        );

        assert_eq!(
            StepDirection::NorthWest.offset(),
            (-1, 1),
            "NorthWest must move negatively on X and positively on Y"
        );
    }

    #[test]
    fn should_saturate_position_math_at_grid_bounds() {
        let origin = Position { x: 0, y: 0 };
        let max = Position {
            x: u16::MAX,
            y: u16::MAX,
        };

        assert_eq!(
            origin + StepDirection::SouthWest,
            origin,
            "Subtracting from zero coordinates should saturate at the lower grid bound"
        );

        assert_eq!(
            max + StepDirection::NorthEast,
            max,
            "Adding beyond u16::MAX should saturate at the upper grid bound"
        );

        assert_eq!(
            origin - StepDirection::NorthEast,
            origin,
            "Subtracting the inverse direction from the lower bound should also saturate"
        );
    }

    #[test]
    fn should_apply_add_assign_and_sub_assign_like_direction_arithmetic() {
        let mut position = Position { x: 10, y: 10 };

        position += StepDirection::NorthWest;

        assert_eq!(
            position,
            Position { x: 9, y: 11 },
            "AddAssign should apply the same coordinate math as Add"
        );

        position -= StepDirection::NorthWest;

        assert_eq!(
            position,
            Position { x: 10, y: 10 },
            "SubAssign should reverse the effect of AddAssign for the same direction"
        );
    }

    #[test]
    fn should_reverse_add_and_sub_for_matching_direction() {
        let start = Position { x: 12, y: 9 };
        let stepped = start + StepDirection::SouthEast;

        assert_eq!(
            stepped - StepDirection::SouthEast,
            start,
            "Subtracting the same direction after stepping should restore the original position \
             when no saturation occurs"
        );
    }
}
