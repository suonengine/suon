//! Chunk-local terrain navigation data.
//!
//! This module models whether registered terrain nodes are temporarily blocked,
//! but it is currently independent from the runtime occupancy sync used by
//! [`crate::ChunkPlugin`].

use bevy::prelude::*;
use enumflags2::{BitFlags, bitflags};
use std::collections::*;
use suon_position::{floor::Floor, position::Position};

pub mod occupied;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NavigationState {
    Registered = 0b0001,
    Occupied = 0b0010,
}

#[derive(Component, Default, Debug)]
/// Passability map for registered terrain nodes within a chunk.
pub struct Navigation {
    nodes: HashMap<(Floor, Position), BitFlags<NavigationState>>,
}

impl Navigation {
    /// Registers a terrain node as part of the navigable map.
    pub fn add_node(&mut self, floor: Floor, position: Position) {
        self.nodes
            .entry((floor, position))
            .and_modify(|flags| *flags |= NavigationState::Registered)
            .or_insert(NavigationState::Registered.into());
    }

    /// Marks a registered node as currently occupied.
    pub fn occupy(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&(floor, position)) {
            *flags |= NavigationState::Occupied;
        }
    }

    /// Releases the occupied state of a registered node.
    pub fn release(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&(floor, position)) {
            flags.remove(NavigationState::Occupied);
        }
    }

    /// Returns whether the node exists in navigation and is currently passable.
    pub fn is_passable(&self, floor: Floor, position: Position) -> bool {
        self.nodes.get(&(floor, position)).map_or(false, |flags| {
            flags.contains(NavigationState::Registered)
                && !flags.contains(NavigationState::Occupied)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_mark_registered_nodes_as_passable() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 0 };
        const POSITION: Position = Position { x: 5, y: 8 };

        // Registered nodes become traversable until explicitly occupied.
        navigation.add_node(FLOOR, POSITION);

        assert!(
            navigation.is_passable(FLOOR, POSITION),
            "A registered node without occupancy should be passable"
        );
    }

    #[test]
    fn should_block_passability_when_registered_node_is_occupied() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 1 };
        const POSITION: Position = Position { x: 9, y: 3 };

        // Occupancy should temporarily disable traversal of an existing node.
        navigation.add_node(FLOOR, POSITION);
        navigation.occupy(FLOOR, POSITION);

        assert!(
            !navigation.is_passable(FLOOR, POSITION),
            "Occupied nodes should not be passable"
        );
    }

    #[test]
    fn should_restore_passability_after_release() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 2 };
        const POSITION: Position = Position { x: 4, y: 4 };

        // Releasing occupancy should return the node to its registered state.
        navigation.add_node(FLOOR, POSITION);
        navigation.occupy(FLOOR, POSITION);
        navigation.release(FLOOR, POSITION);

        assert!(
            navigation.is_passable(FLOOR, POSITION),
            "Releasing a registered node should make it passable again"
        );
    }

    #[test]
    fn should_keep_unregistered_nodes_impassable() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 0 };
        const POSITION: Position = Position { x: 1, y: 1 };

        // Occupy and release should not implicitly create navigable nodes.
        navigation.occupy(FLOOR, POSITION);
        navigation.release(FLOOR, POSITION);

        assert!(
            !navigation.is_passable(FLOOR, POSITION),
            "Unregistered nodes should remain impassable even after occupy/release calls"
        );
    }

    #[test]
    fn should_keep_other_nodes_unchanged_when_one_node_is_occupied_or_released() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 0 };
        const FIRST: Position = Position { x: 1, y: 1 };
        const SECOND: Position = Position { x: 2, y: 2 };

        navigation.add_node(FLOOR, FIRST);
        navigation.add_node(FLOOR, SECOND);
        navigation.occupy(FLOOR, FIRST);
        navigation.release(FLOOR, FIRST);

        assert!(
            navigation.is_passable(FLOOR, FIRST),
            "The released node should become passable again"
        );

        assert!(
            navigation.is_passable(FLOOR, SECOND),
            "Updating one node should not affect the passability of another node"
        );
    }

    #[test]
    fn should_keep_node_blocked_after_repeated_occupy_calls_until_release() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 3 };
        const POSITION: Position = Position { x: 6, y: 6 };

        navigation.add_node(FLOOR, POSITION);
        navigation.occupy(FLOOR, POSITION);
        navigation.occupy(FLOOR, POSITION);

        assert!(
            !navigation.is_passable(FLOOR, POSITION),
            "Repeated occupy calls should leave the node blocked"
        );

        navigation.release(FLOOR, POSITION);

        assert!(
            navigation.is_passable(FLOOR, POSITION),
            "A single release should clear the occupied state after repeated occupy calls"
        );
    }
}
