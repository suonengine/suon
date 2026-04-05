//! Chunk-local navigation passability tracking.
//!
//! [`Navigation`] mirrors occupied floor-position pairs into a passability map
//! stored on each chunk. Known nodes are registered as they are seen by the
//! runtime synchronization flow and are marked blocked while an
//! [`crate::occupancy::occupied::Occupied`] entity is present on that tile.

use crate::{chunks::Chunks, occupancy::occupied::Occupied};
use bevy::prelude::*;
use enumflags2::{BitFlags, bitflags};
use std::collections::HashMap;
use suon_position::{floor::Floor, position::Position, previous_position::PreviousPosition};

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum NavigationState {
    Registered = 0b0001,
    Occupied = 0b0010,
}

#[derive(Component, Default, Debug)]
/// Passability map for known floor-position pairs within a chunk.
pub struct Navigation {
    nodes: HashMap<(Floor, Position), BitFlags<NavigationState>>,
}

impl Navigation {
    /// Registers a floor-position pair as part of the chunk navigation map.
    fn add_node(&mut self, floor: Floor, position: Position) {
        self.nodes
            .entry((floor, position))
            .and_modify(|flags| *flags |= NavigationState::Registered)
            .or_insert(NavigationState::Registered.into());
    }

    /// Marks a registered node as currently occupied.
    fn occupy(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&(floor, position)) {
            *flags |= NavigationState::Occupied;
        }
    }

    /// Releases the occupied state of a registered node.
    fn release(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&(floor, position)) {
            flags.remove(NavigationState::Occupied);
        }
    }

    /// Returns whether the node exists in navigation and is currently passable.
    pub fn is_passable(&self, floor: Floor, position: Position) -> bool {
        self.nodes.get(&(floor, position)).is_some_and(|flags| {
            flags.contains(NavigationState::Registered)
                && !flags.contains(NavigationState::Occupied)
        })
    }
}

/// Registers and blocks a navigation node when an entity gains [`Occupied`].
pub(crate) fn sync_navigation_register(
    event: On<Add, Occupied>,
    entities: Query<(&Position, &Floor)>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor)) = entities.get(entity) else {
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        return;
    };

    if let Ok(mut navigation) = navigation.get_mut(chunk) {
        navigation.add_node(*floor, *position);
        navigation.occupy(*floor, *position);
    }
}

/// Releases the navigation block when an entity loses [`Occupied`].
pub(crate) fn sync_navigation_unregister(
    event: On<Remove, Occupied>,
    entities: Query<(&Position, &Floor, Option<&PreviousPosition>)>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor, previous_position)) = entities.get(entity) else {
        return;
    };

    if let Some(chunk) = chunks.get(position)
        && let Ok(mut navigation) = navigation.get_mut(chunk)
    {
        navigation.release(*floor, *position);
    }

    let Some(previous_position) = previous_position else {
        return;
    };

    let previous_position = Position {
        x: previous_position.x,
        y: previous_position.y,
    };

    if previous_position == *position {
        return;
    }

    if let Some(previous_chunk) = chunks.get(&previous_position)
        && let Ok(mut navigation) = navigation.get_mut(previous_chunk)
    {
        navigation.release(*floor, previous_position);
    }
}

/// Reconciles navigation after an occupied [`Position`] is inserted or replaced.
pub(crate) fn resync_navigation_positions(
    event: On<Insert, Position>,
    entities: Query<(&Position, &PreviousPosition, &Floor), With<Occupied>>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, previous_position, floor)) = entities.get(entity) else {
        return;
    };

    let previous_position = Position {
        x: previous_position.x,
        y: previous_position.y,
    };

    if let Some(previous_chunk) = chunks.get(&previous_position)
        && let Ok(mut navigation) = navigation.get_mut(previous_chunk)
    {
        navigation.release(*floor, previous_position);
    }

    let Some(current_chunk) = chunks.get(position) else {
        return;
    };

    if let Ok(mut navigation) = navigation.get_mut(current_chunk) {
        navigation.add_node(*floor, *position);
        navigation.occupy(*floor, *position);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, ChunkPlugin};

    #[test]
    fn should_mark_registered_nodes_as_passable() {
        let mut navigation = Navigation::default();
        const FLOOR: Floor = Floor { z: 0 };
        const POSITION: Position = Position { x: 5, y: 8 };

        navigation.add_node(FLOOR, POSITION);

        assert!(
            navigation.is_passable(FLOOR, POSITION),
            "A registered node without occupancy should be passable"
        );
    }

    #[test]
    fn should_block_navigation_when_occupied_is_added() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 4, y: 4 }, chunk_entity);

        app.world_mut()
            .spawn((Position { x: 4, y: 4 }, Floor { z: 0 }, Occupied));

        app.update();

        let navigation = app
            .world()
            .get::<Navigation>(chunk_entity)
            .expect("Chunk should carry Navigation");

        assert!(
            !navigation.is_passable(Floor { z: 0 }, Position { x: 4, y: 4 }),
            "Adding Occupied should block the matching navigation node"
        );
    }

    #[test]
    fn should_restore_navigation_when_occupied_is_removed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 9, y: 9 }, chunk_entity);

        let entity = app
            .world_mut()
            .spawn((Position { x: 9, y: 9 }, Floor { z: 1 }, Occupied))
            .id();

        app.update();
        app.world_mut().entity_mut(entity).remove::<Occupied>();
        app.update();

        let navigation = app
            .world()
            .get::<Navigation>(chunk_entity)
            .expect("Chunk should carry Navigation");

        assert!(
            navigation.is_passable(Floor { z: 1 }, Position { x: 9, y: 9 }),
            "Removing Occupied should make the known node passable again"
        );
    }

    #[test]
    fn should_move_navigation_block_when_occupied_entity_moves() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugin);

        let start = Position { x: 7, y: 7 };
        let target = Position { x: 8, y: 7 };
        let start_chunk = app.world_mut().spawn(Chunk).id();
        let target_chunk = app.world_mut().spawn(Chunk).id();

        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&start, start_chunk);
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&target, target_chunk);

        let entity = app
            .world_mut()
            .spawn((start, Floor { z: 0 }, Occupied))
            .id();

        app.update();

        app.world_mut().entity_mut(entity).insert((
            PreviousPosition {
                x: start.x,
                y: start.y,
            },
            target,
        ));

        app.update();

        let start_navigation = app
            .world()
            .get::<Navigation>(start_chunk)
            .expect("Start chunk should carry Navigation");
        let target_navigation = app
            .world()
            .get::<Navigation>(target_chunk)
            .expect("Target chunk should carry Navigation");

        assert!(
            start_navigation.is_passable(Floor { z: 0 }, start),
            "Moving away should release the previous node"
        );

        assert!(
            !target_navigation.is_passable(Floor { z: 0 }, target),
            "Moving onto a target node should block it"
        );
    }
}
