# suon_database

Typed database tables and snapshot persistence for the Suon MMORPG framework.

`suon_database` provides:

- Typed Bevy resources for domain tables
- Focused `SystemParam`s for reading and writing table data
- Snapshot-based persistence contracts for loading and saving tables
- Backend-neutral database connections powered by Bevy task utilities

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_database = { path = "../suon_database" }
suon_macros = { path = "../suon_macros" }
```

## Quick Start

```rust
use bevy::prelude::*;
use suon_database::prelude::*;
use suon_macros::Table;

#[derive(Table, Default)]
struct ItemTable {
    entries: Vec<Item>,
}

#[derive(Clone, Debug)]
struct Item {
    id: u32,
    name: String,
}

fn load_items(mut table: DatabaseMut<ItemTable>) {
    table.entries.push(Item {
        id: 1,
        name: "Sword".into(),
    });
}

fn read_items(table: Database<ItemTable>) {
    for item in &table.entries {
        println!("{}: {}", item.id, item.name);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .init_database_table::<ItemTable>()
        .add_systems(Startup, load_items)
        .add_systems(Update, read_items)
        .run();
}
```

## How It Works

Each type that implements `Table` is stored in its own `Tables<T>` resource. Systems
work with that data through `Database<T>` and `DatabaseMut<T>`, which dereference
directly to the inner table and keep call sites concise.

For persistence, `DatabaseConnection<D>` combines backend-specific data with Bevy task-backed
async execution helpers. `SnapshotTable` defines how a table exposes rows, while `TableMapper<T, D>`
handles backend-specific schema and row translation.

### ECS Tables

| Type | Role |
|---|---|
| `Table` | Marks a type as a database table resource |
| `Tables<T>` | Stores one concrete table as a Bevy resource |
| `Database<T>` | Read-only `SystemParam` for a table |
| `DatabaseMut<T>` | Mutable `SystemParam` for a table |
| `AppTablesExt` | Registers and inserts tables on `App` |

### Snapshot Persistence

| Type | Role |
|---|---|
| `SnapshotTable` | Exposes table rows for persistence |
| `SnapshotTableExt` | Convenience methods for schema, load, and save |
| `TableMapper<T, D>` | Maps a table to a specific backend |
| `DatabaseConnection<D>` | Backend-specific data plus task-backed async helpers |
| `DatabaseData` | Marker trait for backend payloads |
| `PoolData` | Trait for backends that expose a pool handle |
| `DatabasePool` | Default SQL payload backed by `sqlx::AnyPool` |
| `DatabaseSettings` | Connection and pool configuration |

### SQL Connections

Workspace SQL integrations use `DatabaseConnection<DatabasePool>`. A minimal setup looks like this:

```rust
# use anyhow::Result;
# use suon_database::prelude::*;
# fn demo() -> Result<()> {
let settings = DatabaseSettings::default();
let connection = DatabaseConnection::<DatabasePool>::connect(&settings)?;
# let _ = connection;
# Ok(())
# }
```
