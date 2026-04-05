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
