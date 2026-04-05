//! Chunk loading placeholders.
//!
//! The crate already needs a place to store future loading state, so this module
//! exposes a minimal resource that can grow with the loader implementation.

use bevy::prelude::*;

#[derive(Resource, Default)]
/// Resource reserved for future chunk loading orchestration.
pub struct ChunkLoader {}
