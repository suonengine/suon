//! Spatial position components shared across the Suon world.
//!
//! This crate centralizes the lightweight ECS components used to represent where
//! an entity is now and where it was previously, both in two-dimensional world
//! coordinates and across floors.
//!
//! # Modules
//!
//! - [`position`]: current world-space tile coordinates
//! - [`floor`]: current vertical layer
//! - [`previous_position`]: previous world-space tile coordinates
//! - [`previous_floor`]: previous vertical layer

pub mod floor;
pub mod position;
pub mod previous_floor;
pub mod previous_position;

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_position_modules_from_crate_root() {
        let position_name = std::any::type_name::<crate::position::Position>();
        let floor_name = std::any::type_name::<crate::floor::Floor>();
        let previous_position_name =
            std::any::type_name::<crate::previous_position::PreviousPosition>();
        let previous_floor_name = std::any::type_name::<crate::previous_floor::PreviousFloor>();

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
