//! Spatial chunk management for the Suon world.
//!
//! This crate groups world positions into chunk entities and provides the runtime
//! data needed to answer three core spatial questions:
//!
//! - which chunk owns a given world-space
//!   [`suon_position::position::Position`]
//! - which chunk currently contains a given entity
//! - which floor-position pairs inside a chunk are occupied
//!
//! # Responsibilities
//!
//! [`Chunks`] is the global registry that maps world positions to chunk entities.
//! [`Chunk`] marks entities that act as chunk containers and automatically carries
//! an [`Occupancy`] store. [`content::AtChunk`] is the relationship component that
//! links an entity back to the chunk that currently contains it.
//!
//! The crate treats [`suon_position::position::Position`] as the source of truth
//! for chunk ownership. [`ChunkPlugin`] uses lifecycle observers on
//! [`suon_position::position::Position`] to synchronize [`content::AtChunk`] and
//! to resynchronize occupied tiles using
//! [`suon_position::previous_position::PreviousPosition`].
//!
//! # Runtime flow
//!
//! A typical end-to-end flow looks like this:
//!
//! 1. Chunk entities are created with [`Chunk`].
//! 2. The world populates [`Chunks`] with the positions owned by each chunk.
//! 3. Game entities receive a [`suon_position::position::Position`] and optionally
//!    an [`occupancy::occupied::Occupied`] marker.
//! 4. [`ChunkPlugin`] derives [`content::AtChunk`] automatically from the current
//!    [`suon_position::position::Position`].
//! 5. Occupied entities register their current tile in the destination chunk's
//!    [`Occupancy`] map.
//! 6. When an occupied entity moves, the crate releases the previous tile and
//!    registers the new one.
//!
//! # Modules
//!
//! - [`chunks`]: chunk registry and chunk-key derivation
//! - [`content`]: entity-to-chunk relationship components
//! - [`loader`]: placeholder resource for chunk loading orchestration
//! - [`occupancy`]: per-chunk blocked-tile tracking and synchronization
//! - [`terrain`]: per-chunk passability state synchronized from occupied tiles
//!
//! At the moment, the end-to-end runtime flow is centered on [`Chunks`],
//! [`content::AtChunk`], [`Occupancy`], and [`terrain::Navigation`].
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_chunk::{Chunk, ChunkPlugin, chunks::Chunks, content::AtChunk};
//! use suon_position::position::Position;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(ChunkPlugin);
//!
//! let chunk = app.world_mut().spawn(Chunk).id();
//! app.insert_resource(Chunks::from_iter([(Position { x: 4, y: 4 }, chunk)]));
//!
//! let entity = app.world_mut().spawn(Position { x: 4, y: 4 }).id();
//! app.update();
//!
//! let at_chunk = app.world().get::<AtChunk>(entity).unwrap();
//! assert_eq!(at_chunk.entity(), chunk);
//! ```
//!
use crate::{
    chunks::Chunks,
    content::sync_at_chunk_from_position,
    loader::ChunkLoader,
    occupancy::{
        Occupancy, resync_occupied_positions, sync_occupancy_register, sync_occupancy_unregister,
    },
    terrain::{
        Navigation, resync_navigation_positions, sync_navigation_register,
        sync_navigation_unregister,
    },
};
use bevy::prelude::*;

/// Chunk registry and chunk-key utilities.
pub mod chunks;
/// Entity relationship components that connect world content to chunks.
pub mod content;
/// Chunk loading resources and future loading orchestration.
pub mod loader;
/// Occupancy state and synchronization systems for chunk-contained entities.
pub mod occupancy;
/// Terrain navigation data structures synchronized from occupied tiles.
pub mod terrain;
/// The bit exponent used to define the power-of-two dimensions of a chunk.
pub const CHUNK_EXP: usize = 3;

/// The number of units along a single axis of the chunk.
pub const CHUNK_SIZE: usize = 1 << CHUNK_EXP;

/// The total number of tiles contained within the chunk.
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

/// A bitmask used to extract local coordinates from global positions.
pub const CHUNK_MASK: usize = CHUNK_SIZE - 1;

/// Plugin responsible for chunk resources and chunk-local synchronization.
pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Chunks>().init_resource::<ChunkLoader>();

        app.add_observer(sync_occupancy_register)
            .add_observer(sync_occupancy_unregister)
            .add_observer(sync_at_chunk_from_position)
            .add_observer(resync_occupied_positions)
            .add_observer(sync_navigation_register)
            .add_observer(sync_navigation_unregister)
            .add_observer(resync_navigation_positions);
    }
}

#[derive(Component)]
#[component(immutable)]
/// Marker component for chunks or entities that are currently active.
pub struct Active;

#[derive(Component)]
#[component(immutable)]
/// Marker component for chunks or entities that are currently inactive.
pub struct Inactive;

#[derive(Component)]
#[component(immutable)]
#[require(Occupancy, Navigation)]
/// Marker component identifying an entity as a chunk container.
pub struct Chunk;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{AtChunk, Content};
    use suon_position::position::Position;

    #[test]
    fn should_expose_consistent_chunk_constants() {
        // The derived constants must stay aligned with the chunk exponent.
        assert_eq!(
            CHUNK_EXP, 3,
            "Chunk exponent should remain the configured value"
        );

        assert_eq!(
            CHUNK_SIZE,
            1 << CHUNK_EXP,
            "Chunk size should be derived from the exponent"
        );

        assert_eq!(
            CHUNK_AREA,
            CHUNK_SIZE * CHUNK_SIZE,
            "Chunk area should match the square of the edge size"
        );

        assert_eq!(
            CHUNK_MASK,
            CHUNK_SIZE - 1,
            "Chunk mask should cover local coordinates inside one chunk"
        );
    }

    #[test]
    fn should_initialize_chunk_resources_when_plugin_is_added() {
        let mut app = App::new();

        // Adding the plugin should prepare the runtime resources it depends on.
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

        assert!(
            app.world().contains_resource::<Chunks>(),
            "ChunkPlugin should initialize the chunk registry resource"
        );

        assert!(
            app.world().contains_resource::<ChunkLoader>(),
            "ChunkPlugin should initialize the chunk loader resource"
        );
    }

    #[test]
    fn should_require_occupancy_when_spawning_chunk_component() {
        let mut world = World::new();

        // The Chunk component contract guarantees an Occupancy store is present.
        let entity = world.spawn(Chunk).id();

        assert!(
            world.entity(entity).contains::<Occupancy>(),
            "Spawning Chunk should automatically attach Occupancy"
        );
    }

    #[test]
    fn should_require_navigation_when_spawning_chunk_component() {
        let mut world = World::new();

        let entity = world.spawn(Chunk).id();

        assert!(
            world.entity(entity).contains::<Navigation>(),
            "Spawning Chunk should automatically attach Navigation"
        );
    }

    #[test]
    fn should_spawn_active_and_inactive_marker_components() {
        let mut world = World::new();
        let active = world.spawn(Active).id();
        let inactive = world.spawn(Inactive).id();

        assert!(
            world.entity(active).contains::<Active>(),
            "Active should behave like a plain marker component when spawned"
        );

        assert!(
            world.entity(inactive).contains::<Inactive>(),
            "Inactive should behave like a plain marker component when spawned"
        );
    }

    #[test]
    fn should_link_content_back_to_chunk_through_relationship() {
        let mut world = World::new();
        let chunk = world.spawn_empty().id();

        // Spawning a linked content entity should update both sides of the relationship.
        let entity = world.spawn(AtChunk::new(chunk)).id();

        let at_chunk = world
            .get::<AtChunk>(entity)
            .expect("Entity should contain the AtChunk relationship");
        let content = world
            .get::<Content>(chunk)
            .expect("Chunk should receive the Content relationship target");

        assert_eq!(
            at_chunk.entity(),
            chunk,
            "The relationship should preserve the target chunk entity"
        );

        assert!(
            content.contains(&entity),
            "The chunk content list should include linked entities"
        );
    }

    #[test]
    fn should_sync_at_chunk_from_position_automatically() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

        let chunk = app.world_mut().spawn(Chunk).id();
        app.insert_resource(Chunks::from_iter([(Position { x: 4, y: 4 }, chunk)]));

        let entity = app.world_mut().spawn(Position { x: 4, y: 4 }).id();

        app.update();

        let at_chunk = app
            .world()
            .get::<AtChunk>(entity)
            .expect("Position synchronization should assign AtChunk automatically");

        assert_eq!(
            at_chunk.entity(),
            chunk,
            "The derived chunk relationship should match the chunk registry"
        );
    }
}
