//! Spatial position components shared across the Suon world.
//!
//! This crate centralizes the lightweight ECS components used to represent where
//! an entity is now and where it was previously, both in two-dimensional world
//! coordinates and across floors.
//!
//! # Modules
//!
//! - [`direction`]: shared cardinal and diagonal facing/movement directions
//! - [`position`]: current world-space tile coordinates
//! - [`floor`]: current vertical layer
//! - [`previous_position`]: previous world-space tile coordinates
//! - [`previous_floor`]: previous vertical layer
//!
//! # Examples
//! ```
//! use suon_position::{
//!     direction::Direction,
//!     floor::Floor,
//!     position::Position,
//!     previous_floor::PreviousFloor,
//!     previous_position::PreviousPosition,
//! };
//!
//! let position = Position { x: 12, y: 34 };
//! let next_position = position + Direction::East;
//! let floor = Floor { z: 7 };
//! let previous_position = PreviousPosition { x: 11, y: 34 };
//! let previous_floor = PreviousFloor { z: 6 };
//!
//! assert_eq!(position.x, 12);
//! assert_eq!(next_position.x, 13);
//! assert_eq!(*floor, 7);
//! assert_eq!(previous_position.y, 34);
//! assert_eq!(*previous_floor, 6);
//! ```

pub mod direction;
pub mod floor;
pub mod position;
pub mod previous_floor;
pub mod previous_position;

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_position_modules_from_crate_root() {
        let direction_name = std::any::type_name::<crate::direction::Direction>();
        let position_name = std::any::type_name::<crate::position::Position>();
        let floor_name = std::any::type_name::<crate::floor::Floor>();
        let previous_position_name =
            std::any::type_name::<crate::previous_position::PreviousPosition>();
        let previous_floor_name = std::any::type_name::<crate::previous_floor::PreviousFloor>();

        assert!(
            direction_name.contains("direction::Direction"),
            "The direction module should stay publicly accessible from the crate root"
        );
        assert!(
            position_name.contains("position::Position"),
            "The position module should stay publicly accessible from the crate root"
        );
        assert!(
            floor_name.contains("floor::Floor"),
            "The floor module should stay publicly accessible from the crate root"
        );
        assert!(
            previous_position_name.contains("previous_position::PreviousPosition"),
            "The previous_position module should stay publicly accessible from the crate root"
        );
        assert!(
            previous_floor_name.contains("previous_floor::PreviousFloor"),
            "The previous_floor module should stay publicly accessible from the crate root"
        );
    }
}
