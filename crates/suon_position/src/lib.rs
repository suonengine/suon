//! Spatial position components shared across the Suon world.
//!
//! This crate centralizes the lightweight ECS components used to represent where
//! an entity is now and where it was previously, both in two-dimensional world
//! coordinates and across floors.
//!
//! # Modules
//!
//! - [`crate::prelude::Direction`]: shared cardinal and diagonal facing/movement directions
//! - [`crate::prelude::Position`]: current world-space tile coordinates
//! - [`crate::prelude::Floor`]: current vertical layer
//! - [`crate::prelude::PreviousPosition`]: previous world-space tile coordinates
//! - [`crate::prelude::PreviousFloor`]: previous vertical layer
//!
//! # Examples
//! ```
//! use suon_position::prelude::*;
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

mod direction;
mod floor;
mod position;
mod previous_floor;
mod previous_position;

pub mod prelude {
    pub use crate::{
        direction::Direction, floor::Floor, position::Position, previous_floor::PreviousFloor,
        previous_position::PreviousPosition,
    };
}

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

    #[test]
    fn should_expose_position_types_through_prelude() {
        use crate::prelude::*;

        let _ = std::mem::size_of::<Direction>();
        let _ = std::mem::size_of::<Floor>();
        let _ = std::mem::size_of::<Position>();
        let _ = std::mem::size_of::<PreviousFloor>();
        let _ = std::mem::size_of::<PreviousPosition>();
    }
}
