//! Chunk-local navigation passability tracking.
//!
//! [`Navigation`] mirrors occupied floor-position pairs into a passability map
//! stored on each chunk. Known nodes are registered as they are seen by the
//! runtime synchronization flow and are marked blocked while an
//! [`crate::prelude::Occupied`] entity is present on that tile.

use crate::{chunks::Chunks, floor_position_key::FloorPositionKey, occupancy::occupied::Occupied};
use bevy::prelude::*;
use enumflags2::{BitFlags, bitflags};
use std::collections::HashMap;
use suon_position::prelude::*;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// Bit flags that describe the navigation state of a known floor-position node.
enum NavigationState {
    /// The node has been discovered and can participate in passability checks.
    Registered = 0b0001,

    /// The node is currently blocked by an [`Occupied`] entity.
    Occupied = 0b0010,
}

/// Plugin responsible for synchronizing chunk-local [`Navigation`] blocks.
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        info!("Starting the terrain navigation systems");

        app.add_observer(register_navigation_block)
            .add_observer(release_navigation_block)
            .add_observer(reconcile_navigation_position_change)
            .add_observer(reconcile_navigation_floor_change);
    }
}

#[derive(Component, Default, Debug)]
/// Passability map for known floor-position pairs within a chunk.
pub struct Navigation {
    nodes: HashMap<FloorPositionKey, BitFlags<NavigationState>>,
}

impl Navigation {
    /// Registers a floor-position pair as part of the chunk navigation map.
    ///
    /// Registering a node does not mark it occupied; it only makes it known to
    /// [`Navigation::is_passable`].
    fn add_node(&mut self, floor: Floor, position: Position) {
        self.nodes
            .entry(FloorPositionKey::new(floor, position))
            .and_modify(|flags| *flags |= NavigationState::Registered)
            .or_insert(NavigationState::Registered.into());
    }

    /// Marks a registered node as currently occupied.
    ///
    /// Unknown nodes are ignored so occupancy cannot implicitly create passable
    /// terrain without a prior registration step.
    fn occupy(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&FloorPositionKey::new(floor, position)) {
            *flags |= NavigationState::Occupied;
        }
    }

    /// Releases the occupied state of a registered node.
    ///
    /// Unknown nodes are ignored.
    fn release(&mut self, floor: Floor, position: Position) {
        if let Some(flags) = self.nodes.get_mut(&FloorPositionKey::new(floor, position)) {
            flags.remove(NavigationState::Occupied);
        }
    }

    /// Returns whether the node exists in navigation and is currently passable.
    ///
    /// # Examples
    /// ```no_run
    /// use bevy::prelude::*;
    /// use suon_chunk::prelude::*;
    /// use suon_position::prelude::*;
    ///
    /// let mut app = App::new();
    /// app.add_plugins(MinimalPlugins);
    /// app.add_plugins(ChunkPlugins);
    ///
    /// let chunk = app.world_mut().spawn(Chunk).id();
    /// app.insert_resource(Chunks::from_iter([(Position { x: 4, y: 4 }, chunk)]));
    ///
    /// app.world_mut().spawn((Position { x: 4, y: 4 }, Floor { z: 0 }, Occupied));
    /// app.update();
    ///
    /// let navigation = app.world().get::<Navigation>(chunk).unwrap();
    /// assert!(!navigation.is_passable(Floor { z: 0 }, Position { x: 4, y: 4 }));
    /// ```
    pub fn is_passable(&self, floor: Floor, position: Position) -> bool {
        self.nodes
            .get(&FloorPositionKey::new(floor, position))
            .is_some_and(|flags| {
                flags.contains(NavigationState::Registered)
                    && !flags.contains(NavigationState::Occupied)
            })
    }
}

/// Registers and blocks a navigation node when an entity gains [`Occupied`].
pub(crate) fn register_navigation_block(
    event: On<Add, Occupied>,
    entities: Query<(&Position, &Floor)>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor)) = entities.get(entity) else {
        debug!("Skipping navigation registration for {entity:?}: missing Position or Floor");
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        warn!(
            "Skipping navigation registration for {entity:?}: position {position:?} has no chunk"
        );
        return;
    };

    if let Ok(mut navigation) = navigation.get_mut(chunk) {
        navigation.add_node(*floor, *position);
        navigation.occupy(*floor, *position);

        trace!(
            "Registered navigation block for {entity:?} at {:?} on floor {:?} in chunk {:?}",
            position, floor, chunk
        );
    }
}

/// Releases the navigation block when an entity loses [`Occupied`].
pub(crate) fn release_navigation_block(
    event: On<Remove, Occupied>,
    entities: Query<(
        &Position,
        &Floor,
        Option<&PreviousPosition>,
        Option<&PreviousFloor>,
    )>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor, previous_position, previous_floor)) = entities.get(entity) else {
        debug!("Skipping navigation release for {entity:?}: missing Position or Floor");
        return;
    };

    if let Some(chunk) = chunks.get(position)
        && let Ok(mut navigation) = navigation.get_mut(chunk)
    {
        navigation.release(*floor, *position);

        trace!(
            "Released current navigation block for {entity:?} at {:?} on floor {:?} in chunk {:?}",
            position, floor, chunk
        );
    }

    let Some(previous_position) = previous_position else {
        return;
    };

    let previous_position = Position {
        x: previous_position.x,
        y: previous_position.y,
    };

    let previous_floor = previous_floor
        .map(|previous_floor| Floor {
            z: previous_floor.z,
        })
        .unwrap_or(*floor);

    if previous_position == *position && previous_floor == *floor {
        return;
    }

    if let Some(previous_chunk) = chunks.get(&previous_position)
        && let Ok(mut navigation) = navigation.get_mut(previous_chunk)
    {
        navigation.release(previous_floor, previous_position);

        trace!(
            "Released previous navigation block for {entity:?} at {:?} on floor {:?} in chunk {:?}",
            previous_position, previous_floor, previous_chunk
        );
    }
}

/// Reconciles navigation after an occupied [`Position`] is inserted or replaced.
pub(crate) fn reconcile_navigation_position_change(
    event: On<Insert, Position>,
    entities: Query<(&Position, &PreviousPosition, &Floor, Option<&PreviousFloor>), With<Occupied>>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, previous_position, floor, previous_floor)) = entities.get(entity) else {
        debug!(
            "Skipping navigation position reconciliation for {entity:?}: missing movement state"
        );
        return;
    };

    let previous_position = Position {
        x: previous_position.x,
        y: previous_position.y,
    };

    let previous_floor = previous_floor
        .map(|previous_floor| Floor {
            z: previous_floor.z,
        })
        .unwrap_or(*floor);

    if let Some(previous_chunk) = chunks.get(&previous_position)
        && let Ok(mut navigation) = navigation.get_mut(previous_chunk)
    {
        navigation.release(previous_floor, previous_position);

        trace!(
            "Reconciled navigation move for {entity:?}: released {:?} on floor {:?} in chunk {:?}",
            previous_position, previous_floor, previous_chunk
        );
    }

    let Some(current_chunk) = chunks.get(position) else {
        warn!(
            "Skipping navigation target registration for {entity:?}: position {position:?} has no \
             chunk"
        );
        return;
    };

    if let Ok(mut navigation) = navigation.get_mut(current_chunk) {
        navigation.add_node(*floor, *position);
        navigation.occupy(*floor, *position);

        trace!(
            "Reconciled navigation move for {entity:?}: blocked {:?} on floor {:?} in chunk {:?}",
            position, floor, current_chunk
        );
    }
}

/// Reconciles navigation after an occupied [`Floor`] is inserted or replaced.
pub(crate) fn reconcile_navigation_floor_change(
    event: On<Insert, Floor>,
    entities: Query<(&Position, &Floor, Option<&PreviousFloor>), With<Occupied>>,
    mut navigation: Query<&mut Navigation>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor, previous_floor)) = entities.get(entity) else {
        debug!(
            "Skipping navigation floor reconciliation for {entity:?}: missing Position or Floor"
        );
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        warn!(
            "Skipping navigation floor reconciliation for {entity:?}: position {position:?} has \
             no chunk"
        );
        return;
    };

    let Ok(mut navigation) = navigation.get_mut(chunk) else {
        return;
    };

    if let Some(previous_floor) = previous_floor {
        navigation.release(
            Floor {
                z: previous_floor.z,
            },
            *position,
        );

        trace!(
            "Reconciled navigation floor change for {entity:?}: released floor {:?} at {:?} in \
             chunk {:?}",
            previous_floor, position, chunk
        );
    }

    navigation.add_node(*floor, *position);
    navigation.occupy(*floor, *position);

    trace!(
        "Reconciled navigation floor change for {entity:?}: blocked floor {:?} at {:?} in chunk \
         {:?}",
        floor, position, chunk
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, ChunkPlugins};

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
        app.add_plugins(ChunkPlugins);

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
        app.add_plugins(ChunkPlugins);

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
    fn should_reconcile_navigation_block_when_occupied_entity_moves() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

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

    #[test]
    fn should_reconcile_navigation_block_when_occupied_entity_changes_floor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let position = Position { x: 7, y: 7 };
        let chunk_entity = app.world_mut().spawn(Chunk).id();

        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&position, chunk_entity);

        let entity = app
            .world_mut()
            .spawn((position, Floor { z: 0 }, Occupied))
            .id();

        app.update();

        app.world_mut().entity_mut(entity).insert((
            PreviousFloor { z: 0 },
            PreviousPosition {
                x: position.x,
                y: position.y,
            },
            Floor { z: 2 },
            position,
        ));

        app.update();

        let navigation = app
            .world()
            .get::<Navigation>(chunk_entity)
            .expect("Chunk should carry Navigation");

        assert!(
            navigation.is_passable(Floor { z: 0 }, position),
            "Changing floors should release the previous navigation block"
        );

        assert!(
            !navigation.is_passable(Floor { z: 2 }, position),
            "Changing floors should block the new floor-position node"
        );
    }

    #[test]
    fn should_keep_unregistered_nodes_impassable() {
        let navigation = Navigation::default();

        assert!(
            !navigation.is_passable(Floor { z: 4 }, Position { x: 99, y: 99 }),
            "Unknown nodes should remain impassable until they are registered by runtime sync"
        );
    }
}
