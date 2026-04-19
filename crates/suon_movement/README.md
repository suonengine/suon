# suon_movement

Entity movement and grid navigation for the Suon MMORPG framework.

`suon_movement` provides:

- Intent-driven step and teleport components
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
use suon_movement::prelude::*;
use suon_position::prelude::*;

fn request_step(mut commands: Commands, entity: Entity) {
    commands.entity(entity).insert(StepIntent {
        direction: Direction::North,
    });
}

fn on_step(mut reader: EventReader<Step>) {
    for step in reader.read() {
        println!("Entity {:?} stepped to {:?}", step.entity, step.position);
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, MovementPlugins))
        .add_systems(Update, (request_step, on_step))
        .run();
}
```

## How It Works

Movement in Suon is **intent-driven**: instead of modifying `Position` directly, systems
insert an intent component. `MovementPlugins` processes intents each tick, validates them
against chunk occupancy, and emits the corresponding events.

This separation lets downstream systems react to movement through events without coupling
to the internal state machine.

### Stepping

| Component / Event | Role |
|---|---|
| `StepIntent` | Request a single tile step in a `Direction` |
| `StepPath` | Multi-step path; steps are consumed one per tick |
| `Step` | Emitted when a step completes within the same chunk |
| `StepAcrossChunk` | Emitted when a step crosses a chunk boundary |

### Teleporting

| Component / Event | Role |
|---|---|
| `TeleportIntent` | Request an instant jump to a target `Position` |
| `Teleport` | Emitted when a teleport completes within the same chunk |
| `TeleportAcrossChunk` | Emitted when a teleport crosses a chunk boundary |

`Position` is updated and `PreviousPosition` is recorded before any event fires, so
listeners always see the final state.
