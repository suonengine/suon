# suon_chunk

World chunk and map-grid primitives for the Suon MMORPG framework.

`suon_chunk` provides:

- A global registry mapping world positions to chunk entities
- Per-chunk blocked-tile tracking via `Occupancy`
- Passability queries via `Navigation`
- Lifecycle observers that keep chunk membership and occupied tiles in sync with `Position`

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

fn spawn_entity(mut commands: Commands) {
    commands.spawn((
        Position { x: 128, y: 64 },
        Floor { z: 7 },
    ));
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, ChunkPlugin))
        .add_systems(Startup, spawn_entity)
        .run();
}
```

Entities with a `Position` component are placed into the correct `Chunk` automatically —
no manual chunk assignment needed.

## How It Works

The world is divided into fixed-size grid regions called chunks. The `Chunks` resource maps
every occupied world position to the entity that owns that chunk. Each chunk entity carries:

- `Chunk` — marker component identifying the entity as a chunk container
- `Occupancy` — tracks which tiles within the chunk are blocked

When an entity's `Position` changes, lifecycle observers:

1. Remove the entity from its previous chunk via `PreviousPosition`
2. Look up or create the target chunk in `Chunks`
3. Insert/update `AtChunk` on the entity to point at its new chunk
4. Synchronize the blocked-tile set in `Occupancy`

## Core Types

| Type | Role |
|---|---|
| `Chunks` | Global resource mapping world positions to chunk entities |
| `Chunk` | Marker component on chunk container entities |
| `AtChunk` | Relationship component linking an entity to its containing chunk |
| `Occupancy` | Per-chunk blocked-tile store |
| `Navigation` | Per-chunk passability query helper |
