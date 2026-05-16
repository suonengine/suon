# suon_uuid

UUID helpers for the Suon MMORPG framework.

`suon_uuid` provides:

- `Uuid` - re-export of `uuid::Uuid`
- `UuidGenerator` - Bevy resource for generating UUIDs
- `UuidPlugin` - Bevy plugin that registers `UuidGenerator`

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_uuid = { path = "../suon_uuid" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_uuid::prelude::*;

fn spawn_entity(mut commands: Commands, generator: Res<UuidGenerator>) {
    commands.spawn(Name::new(generator.generate().to_string()));
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, UuidPlugin))
        .add_systems(Startup, spawn_entity)
        .run();
}
```

## How It Works

`UuidGenerator` creates UUID v7 values through `Uuid::now_v7()`, keeping generated
IDs time-ordered while still globally unique.

Use `UuidGenerator::generate_uuid()` when an ECS resource is not available.
