//! Occupancy marker components.

use bevy::prelude::*;

#[derive(Component)]
#[component(immutable)]
/// Marker indicating that an entity should block occupancy in its current tile.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use suon_chunk::occupancy::occupied::Occupied;
///
/// let mut world = World::new();
/// let entity = world.spawn(Occupied).id();
///
/// assert!(world.entity(entity).contains::<Occupied>());
/// ```
pub struct Occupied;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_spawn_occupied_marker_component() {
        let mut world = World::new();
        let entity = world.spawn(Occupied).id();

        assert!(
            world.entity(entity).contains::<Occupied>(),
            "Occupied should behave like a plain marker component when spawned"
        );
    }
}
