//! Occupancy marker components.

use bevy::prelude::*;

#[derive(Component)]
#[component(immutable)]
/// Marker indicating that an entity should block occupancy in its current tile.
pub struct Occupied;
