//! Shared world directions and position arithmetic helpers.

use std::{
    cmp::*,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::position::Position;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
/// One-tile direction shared across movement, facing, and packet decoding.
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    /// Returns the `(dx, dy)` offset represented by this direction.
    ///
    /// # Examples
    /// ```
    /// use suon_position::prelude::*;
    ///
    /// assert_eq!(Direction::North.offset(), (0, 1));
    /// assert_eq!(Direction::SouthWest.offset(), (-1, -1));
    /// ```
    pub const fn offset(self) -> (isize, isize) {
        match self {
            Direction::North => (0, 1),
            Direction::NorthEast => (1, 1),
            Direction::East => (1, 0),
            Direction::SouthEast => (1, -1),
            Direction::South => (0, -1),
            Direction::SouthWest => (-1, -1),
            Direction::West => (-1, 0),
            Direction::NorthWest => (-1, 1),
        }
    }
}

impl Add<Direction> for Position {
    type Output = Position;

    fn add(self, direction: Direction) -> Self::Output {
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

impl Sub<Direction> for Position {
    type Output = Position;

    fn sub(self, direction: Direction) -> Self::Output {
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

impl AddAssign<Direction> for Position {
    fn add_assign(&mut self, direction: Direction) {
        *self = *self + direction;
    }
}

impl SubAssign<Direction> for Position {
    fn sub_assign(&mut self, direction: Direction) {
        *self = *self - direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_compare_directions_for_equality_and_uniqueness() {
        const DIRECTION_NORTH: Direction = Direction::North;
        const DIRECTION_NORTH_WEST: Direction = Direction::NorthWest;

        assert_eq!(
            DIRECTION_NORTH,
            Direction::North,
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
            Direction::NorthWest,
            Direction::South,
            Direction::NorthEast,
            Direction::North,
        ];

        directions.sort();

        let expected_order = vec![
            Direction::North,
            Direction::NorthEast,
            Direction::South,
            Direction::NorthWest,
        ];

        assert_eq!(
            directions, expected_order,
            "The directions should be sorted according to their clockwise declaration order"
        );
    }

    #[test]
    fn should_expose_expected_coordinate_offsets() {
        assert_eq!(
            Direction::North.offset(),
            (0, 1),
            "North must move exactly one unit up on the Y axis"
        );

        assert_eq!(
            Direction::NorthWest.offset(),
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
            origin + Direction::SouthWest,
            origin,
            "Subtracting from zero coordinates should saturate at the lower grid bound"
        );

        assert_eq!(
            max + Direction::NorthEast,
            max,
            "Adding beyond u16::MAX should saturate at the upper grid bound"
        );
    }
}
