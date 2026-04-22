# suon_chunk

World chunk and map-grid primitives for the Suon MMORPG framework.

`suon_chunk` provides:

- A global registry mapping world positions to chunk entities
- Per-chunk blocked-tile tracking via `Occupancy`
- Passability queries via `Navigation`
- Lifecycle observers that keep chunk membership, occupancy, and navigation in sync
- Rejection events when chunk relationships cannot be derived

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_chunk = { path = "../suon_chunk" }
suon_position = { path = "../suon_position" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_chunk::prelude::*;
use suon_position::prelude::*;

fn setup(mut commands: Commands) {
    let chunk = commands.spawn(Chunk).id();
    let position = Position { x: 128, y: 64 };

    commands.insert_resource(Chunks::from_iter([(position, chunk)]));
    commands.spawn((position, Floor { z: 7 }));
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, ChunkPlugins))
        .add_systems(Startup, setup)
        .run();
}
```

Entities with a `Position` component are linked to the correct registered `Chunk`
automatically; no manual `AtChunk` assignment is needed.

## How It Works

The world is divided into fixed-size grid regions called chunks. The `Chunks` resource maps
world positions to the entity that owns that chunk. Each chunk entity carries:

- `Chunk` - marker component identifying the entity as a chunk container
- `Occupancy` - tracks which floor-position pairs within the chunk are blocked
- `Navigation` - tracks known passability state for floor-position pairs

`ChunkPlugins` installs smaller context plugins:

- `ChunksPlugin` - initializes `Chunks` and `ChunkLoader`
- `ContentPlugin` - derives `AtChunk` from `Position`
- `OccupancyPlugin` - reconciles occupied floor-position pairs
- `TerrainPlugin` - reconciles navigation/passability blocks

When an entity's `Position` changes, lifecycle observers:

1. Resolve the target chunk from `Chunks`
2. Insert/update `AtChunk` on the entity
3. Emit `AtChunkUpdateRejected` when no registered chunk owns the position
4. Synchronize `Occupancy` and `Navigation` for moved or floor-changing occupied entities

## Core Types

| Type | Role |
|---|---|
| `Chunks` | Global resource mapping world positions to chunk entities |
| `Chunk` | Marker component on chunk container entities |
| `AtChunk` | Relationship component linking an entity to its containing chunk |
| `AtChunkUpdateRejected` | Event emitted when `AtChunk` cannot be derived |
| `Occupancy` | Per-chunk blocked-tile store |
| `Navigation` | Per-chunk passability query helper |
