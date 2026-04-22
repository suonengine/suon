# suon_movement

Entity movement and grid navigation for the Suon MMORPG framework.

`suon_movement` provides:

- Intent-driven step and teleport flows
- Success and rejection events for movement requests
- Events for movement within a chunk and across chunk boundaries
- Path definitions for multi-step movement sequences
- A `MovementPlugins` plugin group that wires everything into the Bevy schedule

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_movement = { path = "../suon_movement" }
suon_position = { path = "../suon_position" }
suon_chunk = { path = "../suon_chunk" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_chunk::prelude::*;
use suon_movement::prelude::*;
use suon_position::prelude::*;

fn setup(mut commands: Commands) {
    let chunk = commands.spawn(Chunk).id();
    let start = Position { x: 1, y: 1 };
    let target = Position { x: 2, y: 1 };
    let entity = commands.spawn((start, Floor { z: 0 })).id();

    commands.insert_resource(Chunks::from_iter([(start, chunk), (target, chunk)]));
    commands.trigger(StepIntent {
        entity,
        to: Direction::East,
    });
}

fn on_step(event: On<Step>) {
    info!("Entity {:?} stepped", event.event_target());
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, ChunkPlugins, MovementPlugins))
        .add_observer(on_step)
        .add_systems(Startup, setup)
        .run();
}
```

## How It Works

Movement in Suon is **intent-driven**: instead of modifying `Position` directly,
systems trigger typed intents. `MovementPlugins` validates each intent against chunk
ownership and occupancy, applies valid changes, and emits success or rejection events.

This separation lets downstream systems react to movement without coupling to the
internal validation flow.

### Stepping

| Component / Event | Role |
|---|---|
| `StepIntent` | Request a single tile step in a `Direction` |
| `StepRejected` | Emitted when a step request cannot be applied |
| `StepPath` | Multi-step path; steps are consumed one per fixed tick |
| `Step` | Emitted when a step updates the entity position |
| `StepAcrossChunk` | Emitted when a step crosses a chunk boundary |

`StepIntent` changes only `Position`. It records `PreviousPosition` before the
position update and leaves `Floor` unchanged.

### Teleporting

| Component / Event | Role |
|---|---|
| `TeleportIntent` | Request an instant jump to a target `Position` and optional `Floor` |
| `TeleportRejected` | Emitted when a teleport request cannot be applied |
| `Teleport` | Emitted when a teleport updates position and/or floor |
| `TeleportAcrossChunk` | Emitted when a teleport crosses a chunk boundary |

`TeleportIntent` can change `Position`, `Floor`, or both. It records
`PreviousPosition` only when position changes and `PreviousFloor` only when floor changes.
