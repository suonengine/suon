//! Chunk-local occupancy tracking.
//!
//! This module keeps track of blocked floor-position pairs inside chunk entities
//! and synchronizes that state from world-space entity movement.

use crate::{chunks::Chunks, floor_position_key::FloorPositionKey, occupancy::occupied::Occupied};
use bevy::prelude::*;
use std::collections::*;
use suon_position::prelude::*;

pub mod occupied;

/// Plugin responsible for synchronizing [`Occupancy`] from occupied entities.
pub struct OccupancyPlugin;

impl Plugin for OccupancyPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(register_occupied_position)
            .add_observer(release_occupied_position)
            .add_observer(reconcile_occupied_position_change)
            .add_observer(reconcile_occupied_floor_change);
    }
}

#[derive(Component, Default, Debug)]
/// Per-chunk occupancy set keyed by compact floor-position pairs.
pub struct Occupancy {
    occupied: HashSet<FloorPositionKey>,
}

impl Occupancy {
    /// Marks the provided floor-position pair as occupied.
    ///
    /// Returns `true` when the pair was newly inserted.
    pub(crate) fn occupy(&mut self, floor: Floor, position: Position) -> bool {
        self.occupied.insert(FloorPositionKey::new(floor, position))
    }

    /// Releases the provided floor-position pair from the occupancy map.
    ///
    /// Returns `true` when the pair was present and has been removed.
    pub(crate) fn release(&mut self, floor: &Floor, position: &Position) -> bool {
        self.occupied
            .remove(&FloorPositionKey::new(*floor, *position))
    }

    /// Returns whether the provided floor-position pair is currently occupied.
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
    /// app.world_mut().spawn((Position { x: 4, y: 4 }, Floor { z: 0 }, Occupied));
    /// app.update();
    ///
    /// let occupancy = app.world().get::<Occupancy>(chunk).unwrap();
    /// assert!(occupancy.contains(&Floor { z: 0 }, &Position { x: 4, y: 4 }));
    /// ```
    pub fn contains(&self, floor: &Floor, position: &Position) -> bool {
        self.occupied
            .contains(&FloorPositionKey::new(*floor, *position))
    }
}

/// Registers chunk occupancy when an entity gains [`Occupied`].
pub(crate) fn register_occupied_position(
    event: On<Add, Occupied>,
    entities: Query<(&Position, &Floor)>,
    mut occupancies: Query<&mut Occupancy>,
    chunks: Res<Chunks>,
) {
    // Occupancy registration follows the chunk resolved from the current world position.
    let entity = event.event_target();

    let Ok((position, floor)) = entities.get(entity) else {
        debug!("Skipping occupancy registration for {entity:?}: missing Position or Floor");
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        warn!("Skipping occupancy registration for {entity:?}: position {position:?} has no chunk");
        return;
    };

    if let Ok(mut occupancy) = occupancies.get_mut(chunk) {
        let changed = occupancy.occupy(*floor, *position);

        trace!(
            "Registered occupancy for {entity:?} at {:?} on floor {:?} in chunk {:?} \
             (changed={changed})",
            position, floor, chunk
        );
    }
}

/// Releases chunk occupancy when an entity loses [`Occupied`].
pub(crate) fn release_occupied_position(
    event: On<Remove, Occupied>,
    entities: Query<(
        &Position,
        &Floor,
        Option<&PreviousPosition>,
        Option<&PreviousFloor>,
    )>,
    mut occupancies: Query<&mut Occupancy>,
    chunks: Res<Chunks>,
) {
    // Removal releases both the current and the previously known coordinate so stale
    // occupancy cannot survive a remove that happens in the same frame as movement.
    let entity = event.event_target();

    let Ok((position, floor, previous_position, previous_floor)) = entities.get(entity) else {
        debug!("Skipping occupancy release for {entity:?}: missing Position or Floor");
        return;
    };

    if let Some(chunk) = chunks.get(position)
        && let Ok(mut occupancy) = occupancies.get_mut(chunk)
    {
        let changed = occupancy.release(floor, position);

        trace!(
            "Released current occupancy for {entity:?} at {:?} on floor {:?} in chunk {:?} \
             (changed={changed})",
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
        && let Ok(mut occupancy) = occupancies.get_mut(previous_chunk)
    {
        let changed = occupancy.release(&previous_floor, &previous_position);

        trace!(
            "Released previous occupancy for {entity:?} at {:?} on floor {:?} in chunk {:?} \
             (changed={changed})",
            previous_position, previous_floor, previous_chunk
        );
    }
}

/// Reconciles chunk occupancy after an occupied [`Position`] is inserted or replaced.
pub(crate) fn reconcile_occupied_position_change(
    event: On<Insert, Position>,
    entities: Query<(&Position, &PreviousPosition, &Floor, Option<&PreviousFloor>), With<Occupied>>,
    mut occupancies: Query<&mut Occupancy>,
    chunks: Res<Chunks>,
) {
    // Moving an occupied entity requires releasing the previous coordinate before
    // registering the new one so chunk occupancy remains authoritative.
    let entity = event.event_target();

    let Ok((position, previous_position, floor, previous_floor)) = entities.get(entity) else {
        debug!("Skipping occupancy position reconciliation for {entity:?}: missing movement state");
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
        && let Ok(mut occupancy) = occupancies.get_mut(previous_chunk)
    {
        let changed = occupancy.release(&previous_floor, &previous_position);

        trace!(
            "Reconciled occupancy move for {entity:?}: released {:?} on floor {:?} in chunk {:?} \
             (changed={changed})",
            previous_position, previous_floor, previous_chunk
        );
    }

    let Some(current_chunk) = chunks.get(position) else {
        warn!(
            "Skipping occupancy target registration for {entity:?}: position {position:?} has no \
             chunk"
        );
        return;
    };

    if let Ok(mut occupancy) = occupancies.get_mut(current_chunk) {
        let changed = occupancy.occupy(*floor, *position);

        trace!(
            "Reconciled occupancy move for {entity:?}: occupied {:?} on floor {:?} in chunk {:?} \
             (changed={changed})",
            position, floor, current_chunk
        );
    }
}

/// Reconciles chunk occupancy after an occupied [`Floor`] is inserted or replaced.
pub(crate) fn reconcile_occupied_floor_change(
    event: On<Insert, Floor>,
    entities: Query<(&Position, &Floor, Option<&PreviousFloor>), With<Occupied>>,
    mut occupancies: Query<&mut Occupancy>,
    chunks: Res<Chunks>,
) {
    let entity = event.event_target();

    let Ok((position, floor, previous_floor)) = entities.get(entity) else {
        debug!("Skipping occupancy floor reconciliation for {entity:?}: missing Position or Floor");
        return;
    };

    let Some(chunk) = chunks.get(position) else {
        warn!(
            "Skipping occupancy floor reconciliation for {entity:?}: position {position:?} has no \
             chunk"
        );
        return;
    };

    let Ok(mut occupancy) = occupancies.get_mut(chunk) else {
        return;
    };

    if let Some(previous_floor) = previous_floor {
        let changed = occupancy.release(
            &Floor {
                z: previous_floor.z,
            },
            position,
        );

        trace!(
            "Reconciled occupancy floor change for {entity:?}: released floor {:?} at {:?} in \
             chunk {:?} (changed={changed})",
            previous_floor, position, chunk
        );
    }

    let changed = occupancy.occupy(*floor, *position);
    trace!(
        "Reconciled occupancy floor change for {entity:?}: occupied floor {:?} at {:?} in chunk \
         {:?} (changed={changed})",
        floor, position, chunk
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Chunk, ChunkPlugins};

    #[test]
    fn should_register_occupied_tile_when_component_is_added() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        // Spawning an Occupied entity should automatically populate the owning chunk map.
        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 4, y: 4 }, chunk_entity);

        app.world_mut()
            .spawn((Position { x: 4, y: 4 }, Floor { z: 0 }, Occupied));

        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            occupancy.contains(&Floor { z: 0 }, &Position { x: 4, y: 4 }),
            "Adding Occupied should mark the tile as occupied in its chunk"
        );
    }

    #[test]
    fn should_unregister_occupied_tile_when_component_is_removed() {
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

        // Removing the marker should release the occupied coordinate on the next update.
        app.update();
        app.world_mut().entity_mut(entity).remove::<Occupied>();
        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 1 }, &Position { x: 9, y: 9 }),
            "Removing Occupied should release the tile from the chunk occupancy"
        );
    }

    #[test]
    fn should_register_occupied_tile_without_manual_at_chunk_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 16, y: 16 }, chunk_entity);

        app.world_mut()
            .spawn((Position { x: 16, y: 16 }, Floor { z: 0 }, Occupied));

        // Occupancy registration should resolve the owner from Position alone.
        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            occupancy.contains(&Floor { z: 0 }, &Position { x: 16, y: 16 }),
            "Sync should not require AtChunk when the chunk can be derived from Position"
        );
    }

    #[test]
    fn should_ignore_registration_when_position_has_no_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .spawn((Position { x: 30, y: 30 }, Floor { z: 2 }, Occupied));

        // If the registry does not know the position, synchronization should no-op.
        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 2 }, &Position { x: 30, y: 30 }),
            "Sync should be a no-op when the position is not mapped in Chunks"
        );
    }

    #[test]
    fn should_track_floors_independently_for_same_position() {
        let mut occupancy = Occupancy::default();
        let tracked_position = Position { x: 2, y: 2 };

        // The same world coordinate can be occupied independently on different floors.
        assert!(
            occupancy.occupy(Floor { z: 0 }, tracked_position),
            "A fresh floor-position pair should be inserted"
        );

        assert!(
            occupancy.occupy(Floor { z: 1 }, tracked_position),
            "The same position on another floor should be tracked independently"
        );

        assert!(
            occupancy.contains(&Floor { z: 0 }, &tracked_position),
            "Floor zero should still contain the position"
        );

        assert!(
            occupancy.contains(&Floor { z: 1 }, &tracked_position),
            "Floor one should also contain the position"
        );

        assert!(
            occupancy.release(&Floor { z: 0 }, &tracked_position),
            "Releasing an occupied position should report success"
        );

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &tracked_position),
            "The released floor should no longer contain the position"
        );

        assert!(
            occupancy.contains(&Floor { z: 1 }, &tracked_position),
            "Other floors should remain untouched by a release"
        );

        assert!(
            !occupancy.release(&Floor { z: 0 }, &tracked_position),
            "Releasing the same position twice should report that nothing changed"
        );
    }

    #[test]
    fn should_report_duplicate_occupy_on_same_floor_and_position() {
        let mut occupancy = Occupancy::default();
        let floor = Floor { z: 0 };
        let position = Position { x: 3, y: 3 };

        assert!(
            occupancy.occupy(floor, position),
            "The first insertion of a floor-position pair should succeed"
        );

        assert!(
            !occupancy.occupy(floor, position),
            "Reinserting the same floor-position pair should report no change"
        );
    }

    #[test]
    fn should_return_false_when_releasing_unknown_floor_or_position() {
        let mut occupancy = Occupancy::default();

        assert!(
            !occupancy.release(&Floor { z: 7 }, &Position { x: 11, y: 11 }),
            "Releasing a floor-position pair that was never occupied should report no change"
        );
    }

    #[test]
    fn should_ignore_registration_when_occupied_entity_is_missing_floor() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 4, y: 4 }, chunk_entity);

        app.world_mut().spawn((Position { x: 4, y: 4 }, Occupied));

        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &Position { x: 4, y: 4 }),
            "Registration should no-op when the occupied entity does not have a Floor"
        );
    }

    #[test]
    fn should_reconcile_occupied_entity_after_position_change_in_same_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 1, y: 1 }, chunk_entity);
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&Position { x: 2, y: 1 }, chunk_entity);

        let entity = app
            .world_mut()
            .spawn((Position { x: 1, y: 1 }, Floor { z: 0 }, Occupied))
            .id();

        // Updating Position with a PreviousPosition should migrate occupancy in place.
        app.update();

        app.world_mut()
            .entity_mut(entity)
            .insert((PreviousPosition { x: 1, y: 1 }, Position { x: 2, y: 1 }));

        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &Position { x: 1, y: 1 }),
            "The old position should be released after a move"
        );

        assert!(
            occupancy.contains(&Floor { z: 0 }, &Position { x: 2, y: 1 }),
            "The new position should be registered after a move"
        );
    }

    #[test]
    fn should_reconcile_occupied_entity_after_cross_chunk_position_change() {
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

        // Cross-chunk movement should release the source chunk and populate the target.
        app.update();

        app.world_mut().entity_mut(entity).insert((
            PreviousPosition {
                x: start.x,
                y: start.y,
            },
            target,
        ));

        app.update();

        let start_occupancy = app
            .world()
            .get::<Occupancy>(start_chunk)
            .expect("Start chunk should carry Occupancy");
        let target_occupancy = app
            .world()
            .get::<Occupancy>(target_chunk)
            .expect("Target chunk should carry Occupancy");

        assert!(
            !start_occupancy.contains(&Floor { z: 0 }, &start),
            "Cross-chunk movement should release the old chunk occupancy"
        );

        assert!(
            target_occupancy.contains(&Floor { z: 0 }, &target),
            "Cross-chunk movement should register occupancy in the target chunk"
        );
    }

    #[test]
    fn should_release_previous_occupancy_even_when_new_position_has_no_registered_chunk() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let start = Position { x: 1, y: 1 };
        let unmapped_target = Position { x: 40, y: 40 };
        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&start, chunk_entity);

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
            unmapped_target,
        ));

        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &start),
            "Moving to an unmapped position should still release the previous occupancy"
        );

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &unmapped_target),
            "Moving to an unmapped position should not register occupancy anywhere"
        );
    }

    #[test]
    fn should_unregister_using_previous_position_when_removed_after_move() {
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
        app.world_mut().entity_mut(entity).remove::<Occupied>();
        app.update();

        let start_occupancy = app
            .world()
            .get::<Occupancy>(start_chunk)
            .expect("Start chunk should carry Occupancy");
        let target_occupancy = app
            .world()
            .get::<Occupancy>(target_chunk)
            .expect("Target chunk should carry Occupancy");

        assert!(
            !start_occupancy.contains(&Floor { z: 0 }, &start),
            "Removing Occupied after a move should not leave stale occupancy in the previous chunk"
        );

        assert!(
            !target_occupancy.contains(&Floor { z: 0 }, &target),
            "Removing Occupied after a move should release the current chunk occupancy too"
        );
    }

    #[test]
    fn should_unregister_current_position_when_previous_position_matches_current() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let position = Position { x: 5, y: 5 };
        let chunk_entity = app.world_mut().spawn(Chunk).id();
        app.world_mut()
            .resource_mut::<Chunks>()
            .insert(&position, chunk_entity);

        let entity = app
            .world_mut()
            .spawn((
                position,
                PreviousPosition {
                    x: position.x,
                    y: position.y,
                },
                Floor { z: 0 },
                Occupied,
            ))
            .id();

        app.update();
        app.world_mut().entity_mut(entity).remove::<Occupied>();
        app.update();

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &position),
            "When previous and current positions match, removal should still release the current \
             occupancy"
        );
    }

    #[test]
    fn should_reconcile_occupied_entity_after_floor_change() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(ChunkPlugins);

        let position = Position { x: 6, y: 6 };
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

        let occupancy = app
            .world()
            .get::<Occupancy>(chunk_entity)
            .expect("Chunk should carry Occupancy");

        assert!(
            !occupancy.contains(&Floor { z: 0 }, &position),
            "Changing floors should release occupancy on the previous floor"
        );

        assert!(
            occupancy.contains(&Floor { z: 2 }, &position),
            "Changing floors should register occupancy on the new floor"
        );
    }

    #[test]
    fn should_treat_other_floors_as_unoccupied() {
        let mut occupancy = Occupancy::default();
        let position = Position { x: 10, y: 10 };

        occupancy.occupy(Floor { z: 2 }, position);

        assert!(
            !occupancy.contains(&Floor { z: 3 }, &position),
            "contains should remain false for floors that were never occupied"
        );
    }
}
