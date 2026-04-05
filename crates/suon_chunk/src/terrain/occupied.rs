//! Terrain occupancy marker components.

use bevy::prelude::*;

#[derive(Component)]
#[component(immutable)]
/// Marker reserved for terrain occupancy semantics within navigation systems.
pub struct Occupied;
