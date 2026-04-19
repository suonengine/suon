# suon_position

Position and coordinate types for the Suon MMORPG framework.

`suon_position` provides:

- `Position` — current world-space tile coordinates
- `Floor` — current vertical layer (z-axis)
- `PreviousPosition` / `PreviousFloor` — coordinates from the previous tick
- `Direction` — cardinal and diagonal facing and movement directions

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_position = { path = "../suon_position" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_position::prelude::*;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Position { x: 100, y: 200 },
        Floor { z: 7 },
    ));
}

fn move_north(mut query: Query<&mut Position>) {
    for mut pos in &mut query {
        *pos = *pos + Direction::North;
    }
}
```

## How It Works

These are plain Bevy `Component` types — no logic lives inside them. Other crates
(`suon_chunk`, `suon_movement`) react to changes via lifecycle observers and events.

### Types

| Type | Fields | Notes |
|---|---|---|
| `Position` | `x: i32`, `y: i32` | Supports `+ Direction` for single-step offsets |
| `Floor` | `z: i32` | Implements `Deref` to `z` |
| `PreviousPosition` | `x: i32`, `y: i32` | Written before any movement event fires |
| `PreviousFloor` | `z: i32` | Written before any movement event fires |
| `Direction` | enum variant | North, South, East, West, and four diagonals |

### Direction

```rust,ignore
let pos = Position { x: 10, y: 10 };
let next = pos + Direction::NorthEast;
// next == Position { x: 11, y: 9 }
```
