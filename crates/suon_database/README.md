# suon_database

Database abstraction layer for the Suon MMORPG framework.

`suon_database` provides:

- A `Table` trait that marks structs as typed database tables
- A `Tables<T>` resource that stores a specific table
- `Database<T>` and `DatabaseMut<T>` system parameters for focused, type-safe read and write access
- An `AppTablesExt` extension for registering tables on startup

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_database = { path = "../suon_database" }
suon_macros = { path = "../suon_macros" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_database::prelude::*;
use suon_macros::Table;

#[derive(Table, Default)]
struct ItemTable {
    entries: Vec<Item>,
}

struct Item {
    id: u32,
    name: String,
}

fn load_items(mut db: DatabaseMut<ItemTable>) {
    db.entries.push(Item { id: 1, name: "Sword".into() });
}

fn read_items(db: Database<ItemTable>) {
    for item in &db.entries {
        println!("{}: {}", item.id, item.name);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_table::<ItemTable>()
        .add_systems(Startup, load_items)
        .add_systems(Update, read_items)
        .run();
}
```

## How It Works

Each table type implementing `Table` is stored in a dedicated `Tables<T>` resource. Systems
access it through `Database<T>` (read-only) or `DatabaseMut<T>` (mutable), which implement
`SystemParam` and dereference directly to the inner table — no boilerplate `Res` unwrapping
needed.

`AppTablesExt::add_table::<T>()` inserts the resource and registers it with the world at
startup.

## Core Types

| Type | Role |
|---|---|
| `Table` | Trait marker for database table structs (derive via `#[derive(Table)]`) |
| `Tables<T>` | Resource holding a single typed table |
| `Database<T>` | Read-only `SystemParam` accessor |
| `DatabaseMut<T>` | Mutable `SystemParam` accessor |
| `AppTablesExt` | Extension trait to register tables on `App` |
