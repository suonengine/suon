//! Chunk ownership registry.
//!
//! This module exposes [`Chunks`], the resource used to resolve which chunk entity
//! owns a given world-space [`Position`].

use crate::chunks::key::ChunkKey;
use bevy::prelude::*;
use std::collections::*;
use suon_position::position::Position;

pub mod key;

#[derive(Resource, Default, Debug)]
/// Global registry mapping world positions to their owning chunk entities.
pub struct Chunks {
    inner: HashMap<ChunkKey, Entity>,
}

impl Chunks {
    /// Resolves the chunk entity responsible for the provided world position.
    pub fn get(&self, position: &Position) -> Option<Entity> {
        self.inner.get(&position.into()).cloned()
    }

    /// Registers the chunk entity responsible for the provided world position.
    pub(crate) fn insert(&mut self, position: &Position, entity: Entity) {
        self.inner.insert(position.into(), entity);
    }

    /// Removes and returns the chunk entity associated with the provided position.
    #[cfg(test)]
    pub(crate) fn remove(&mut self, position: &Position) -> Option<Entity> {
        self.inner.remove(&position.into())
    }

    /// Returns whether the provided world position is mapped to a chunk.
    pub fn contains(&self, position: &Position) -> bool {
        self.inner.contains_key(&position.into())
    }

    /// Returns the number of tracked chunk keys.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether no chunk keys are currently tracked.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }
}

impl FromIterator<(Position, Entity)> for Chunks {
    /// Builds a chunk registry from `(position, chunk_entity)` pairs.
    fn from_iter<T: IntoIterator<Item = (Position, Entity)>>(iter: T) -> Self {
        let mut chunks = Self::default();

        for (position, entity) in iter {
            chunks.insert(&position, entity);
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_store_and_lookup_chunk_by_position() {
        let mut chunks = Chunks::default();
        const CHUNK: Entity = Entity::from_bits(7);
        const POSITION: Position = Position { x: 12, y: 20 };

        // Inserting one mapping should make the same position fully queryable.
        chunks.insert(&POSITION, CHUNK);

        assert_eq!(
            chunks.get(&POSITION),
            Some(CHUNK),
            "The stored chunk entity should be returned for the same position"
        );

        assert!(
            chunks.contains(&POSITION),
            "Inserted positions should be reported as present"
        );

        assert_eq!(chunks.len(), 1, "One inserted chunk should be tracked");

        assert!(
            !chunks.is_empty(),
            "The registry should no longer be empty after insertion"
        );
    }

    #[test]
    fn should_share_same_chunk_mapping_inside_chunk_bounds() {
        let mut chunks = Chunks::default();
        const FIRST_CHUNK: Entity = Entity::from_bits(1);
        const REPLACEMENT_CHUNK: Entity = Entity::from_bits(2);
        const BASE_POSITION: Position = Position { x: 8, y: 16 };
        const SAME_CHUNK_POSITION: Position = Position { x: 15, y: 23 };

        // Positions inside one chunk footprint collapse to the same registry key.
        chunks.insert(&BASE_POSITION, FIRST_CHUNK);
        chunks.insert(&SAME_CHUNK_POSITION, REPLACEMENT_CHUNK);

        assert_eq!(
            chunks.get(&BASE_POSITION),
            Some(REPLACEMENT_CHUNK),
            "Positions in the same chunk key should resolve to the latest mapped entity"
        );

        assert_eq!(
            chunks.get(&SAME_CHUNK_POSITION),
            Some(REPLACEMENT_CHUNK),
            "Both positions should share the same chunk entry"
        );

        assert_eq!(
            chunks.len(),
            1,
            "Two positions inside one chunk should still occupy one registry slot"
        );
    }

    #[test]
    fn should_remove_mapping_and_clear_registry() {
        let mut chunks = Chunks::default();
        const CHUNK: Entity = Entity::from_bits(42);
        const POSITION: Position = Position { x: 0, y: 0 };

        // Removal should return the old mapping and make the position unresolved.
        chunks.insert(&POSITION, CHUNK);

        assert_eq!(
            chunks.remove(&POSITION),
            Some(CHUNK),
            "Removing a registered chunk should return its entity"
        );

        assert!(
            chunks.get(&POSITION).is_none(),
            "Removed positions should no longer resolve to a chunk"
        );

        chunks.insert(&POSITION, CHUNK);
        chunks.clear();

        // The test-only clear helper is useful for resetting registry state in fixtures.
        assert!(chunks.is_empty(), "clear should drop all registered mappings");
    }
}
