//! Spatial chunk management for the Suon world.
//!
//! This crate groups world positions into chunk entities and provides the runtime
//! data needed to answer three core spatial questions:
//!
//! - which chunk owns a given world-space [`suon_position::prelude::Position`]
//! - which chunk currently contains a given entity
//! - which floor-position pairs inside a chunk are occupied
//!
//! # Responsibilities
//!
//! [`crate::prelude::Chunks`] is the global registry that maps world positions to
//! chunk entities. [`Chunk`] marks entities that act as chunk containers and
//! automatically carries an [`crate::prelude::Occupancy`] store.
//! [`crate::prelude::AtChunk`] is the relationship component that links an entity
//! back to the chunk that currently contains it.
//!
//! The crate treats [`suon_position::prelude::Position`] as the source of truth
//! for chunk ownership. [`ChunkPlugins`] uses lifecycle observers on
//! [`suon_position::prelude::Position`] to synchronize [`crate::prelude::AtChunk`]
//! and to resynchronize occupied tiles using
//! [`suon_position::prelude::PreviousPosition`] and
//! [`suon_position::prelude::PreviousFloor`].
//!
//! # Runtime flow
//!
//! A typical end-to-end flow looks like this:
//!
//! 1. Chunk entities are created with [`Chunk`].
//! 2. The world populates [`crate::prelude::Chunks`] with the positions owned by
//!    each chunk.
//! 3. Game entities receive a [`suon_position::prelude::Position`] and optionally
//!    an [`occupancy::occupied::Occupied`] marker.
//! 4. [`ChunkPlugins`] derives [`crate::prelude::AtChunk`] automatically from the
//!    current [`suon_position::prelude::Position`].
//! 5. Occupied entities register their current tile in the destination chunk's
//!    [`crate::prelude::Occupancy`] map.
//! 6. When an occupied entity moves or changes floors, the crate releases the
//!    previous floor-position pair and registers the new one.
//!
//! # Modules
//!
//! - `chunks`: chunk registry and chunk-key derivation
//! - `content`: entity-to-chunk relationship components
//! - `loader`: placeholder resource for chunk loading orchestration
//! - `occupancy`: per-chunk blocked-tile tracking and synchronization
//! - `terrain`: per-chunk passability state synchronized from occupied tiles
//!
//! At the moment, the end-to-end runtime flow is centered on
//! [`crate::prelude::Chunks`], [`crate::prelude::AtChunk`],
//! [`crate::prelude::Occupancy`], and [`crate::prelude::Navigation`].
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_chunk::prelude::*;
//! use suon_position::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(ChunkPlugins);
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
    chunks::ChunksPlugin,
    content::ContentPlugin,
    occupancy::{Occupancy as ChunkOccupancy, OccupancyPlugin},
    terrain::{Navigation as ChunkNavigation, TerrainPlugin},
};
use bevy::{app::PluginGroupBuilder, prelude::*};

/// Chunk registry and chunk-key utilities.
mod chunks;
/// Entity relationship components that connect world content to chunks.
mod content;
/// Compact floor-position keys shared by chunk-local lookup tables.
mod floor_position_key;
/// Chunk loading resources and future loading orchestration.
mod loader;
/// Occupancy state and synchronization systems for chunk-contained entities.
mod occupancy;
/// Terrain navigation data structures synchronized from occupied tiles.
mod terrain;

/// Common chunk types and plugin groups for downstream crates.
pub mod prelude {
    pub use crate::{
        Active, CHUNK_AREA, CHUNK_EXP, CHUNK_MASK, CHUNK_SIZE, Chunk, ChunkPlugins, Inactive,
        chunks::{Chunks, ChunksPlugin},
        content::{AtChunk, AtChunkUpdateError, AtChunkUpdateRejected, Content, ContentPlugin},
        loader::ChunkLoader,
        occupancy::{Occupancy, OccupancyPlugin, occupied::Occupied},
        terrain::{Navigation, TerrainPlugin},
    };
}
/// The bit exponent used to define the power-of-two dimensions of a chunk.
pub const CHUNK_EXP: usize = 3;

/// The number of units along a single axis of the chunk.
pub const CHUNK_SIZE: usize = 1 << CHUNK_EXP;

/// The total number of tiles contained within the chunk.
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

/// A bitmask used to extract local coordinates from global positions.
pub const CHUNK_MASK: usize = CHUNK_SIZE - 1;

/// Plugin group for chunk resources and chunk-local synchronization.
///
/// This installs the smaller chunk context plugins: [`ChunksPlugin`],
/// [`ContentPlugin`], [`OccupancyPlugin`], and [`TerrainPlugin`].
pub struct ChunkPlugins;

impl PluginGroup for ChunkPlugins {
    fn build(self) -> PluginGroupBuilder {
        info!("Loading the chunk systems");

        PluginGroupBuilder::start::<Self>()
            .add(ChunksPlugin)
            .add(ContentPlugin)
            .add(OccupancyPlugin)
            .add(TerrainPlugin)
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
#[require(ChunkOccupancy, ChunkNavigation)]
/// Marker component identifying an entity as a chunk container.
pub struct Chunk;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        content::{AtChunk, Content},
        prelude::*,
    };
    use suon_position::prelude::*;

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
        app.add_plugins(ChunkPlugins);

        assert!(
            app.world().contains_resource::<Chunks>(),
            "ChunkPlugins should initialize the chunk registry resource"
        );

        assert!(
            app.world().contains_resource::<ChunkLoader>(),
            "ChunkPlugins should initialize the chunk loader resource"
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
    fn should_update_at_chunk_after_position_change_automatically() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

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

    #[test]
    fn should_expose_public_chunk_api_through_prelude() {
        use crate::prelude::*;

        let _ = std::mem::size_of::<Active>();
        let _ = std::mem::size_of::<AtChunk>();
        let _ = std::mem::size_of::<AtChunkUpdateError>();
        let _ = std::mem::size_of::<AtChunkUpdateRejected>();
        let _ = std::mem::size_of::<Chunk>();
        let _ = std::mem::size_of::<ChunkLoader>();
        let _ = std::mem::size_of::<ChunkPlugins>();
        let _ = std::mem::size_of::<ChunksPlugin>();
        let _ = std::mem::size_of::<Chunks>();
        let _ = std::mem::size_of::<ContentPlugin>();
        let _ = std::mem::size_of::<Content>();
        let _ = std::mem::size_of::<Inactive>();
        let _ = std::mem::size_of::<OccupancyPlugin>();
        let _ = std::mem::size_of::<Navigation>();
        let _ = std::mem::size_of::<Occupancy>();
        let _ = std::mem::size_of::<Occupied>();
        let _ = std::mem::size_of::<TerrainPlugin>();

        assert_eq!(CHUNK_EXP, 3);
        assert_eq!(CHUNK_SIZE, 1 << CHUNK_EXP);
        assert_eq!(CHUNK_AREA, CHUNK_SIZE * CHUNK_SIZE);
        assert_eq!(CHUNK_MASK, CHUNK_SIZE - 1);
    }
}
