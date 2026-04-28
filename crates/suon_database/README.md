# suon_database

Diesel-backed database integration for Bevy apps in the Suon MMORPG framework.

`suon_database` is the **facilitator** between Bevy systems and a relational
database. It does not hide Diesel — instead it gives every typed table a
predictable shape (`Db<T>` for read access, `DbMut<T>` for writes) and a small
trait surface that hooks the table into the persistence pipeline.

## Highlights

- **One plugin**: `DbPlugin` loads settings and opens the shared connection.
- **One connection type**: `DbConnection`, a Bevy `Resource`.
- **Two persistence flavours**:
  - `DbTable` for snapshot tables (load + bulk save).
  - `DbAppend` for append-only journals (history / audit logs).
- **Auto-dirty tracking**: every `DbMut<T>` mutation bumps the table's dirty
  epoch — no manual `mark_dirty()` calls.

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_database = { path = "../suon_database" }
suon_macros = { path = "../suon_macros" }
```

## In-memory table

```rust
use bevy::prelude::*;
use suon_database::prelude::*;
use suon_macros::Table;

#[derive(Table, Default)]
struct ItemTable {
    items: Vec<Item>,
}

#[derive(Clone, Debug)]
struct Item {
    id: u32,
    name: String,
}

fn add_item(mut items: DbMut<ItemTable>) {
    items.items.push(Item { id: 1, name: "Sword".into() });
}

fn read_items(items: Db<ItemTable>) {
    for item in &items.items {
        println!("{}: {}", item.id, item.name);
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(DbPlugin)
        .init_db_table::<ItemTable>()
        .add_systems(Startup, add_item)
        .add_systems(Update, read_items)
        .run();
}
```

## Snapshot persistence

Implement `DbTable` to wire a table into the load / save pipeline. The
extension trait `init_db_persistent::<T>()` auto-loads on `Startup`, periodic
flushes on `Update`, and drains pending writes on `AppExit`.

```rust,ignore
use anyhow::Result;
use suon_database::prelude::*;
use suon_macros::Table;

#[derive(Table, Default)]
struct ScoreTable { rows: Vec<(i64, String)> }

impl DbTable for ScoreTable {
    type Row = (i64, String);

    fn replace_rows(&mut self, rows: Vec<Self::Row>) { self.rows = rows; }
    fn rows(&self) -> Vec<Self::Row> { self.rows.clone() }

    fn initialize_schema(_: &DbConnection) -> Result<()> { Ok(()) }
    fn load(_: &DbConnection) -> Result<Vec<Self::Row>> { todo!() }
    fn save(_: &DbConnection, _: &[Self::Row]) -> Result<()> { todo!() }
}

App::new()
    .add_plugins(MinimalPlugins)
    .add_plugins(DbPlugin)
    .init_db_persistent::<ScoreTable>();
```

## Append-only journals

For history tables that only insert rows, implement `DbAppend` and register
the schema with `init_db_journal::<T>()`. Insert calls go through
`T::append(&connection, &row)` directly inside your systems / observers.

```rust,ignore
use anyhow::Result;
use suon_database::prelude::*;

struct LoginAuditEntry { actor_id: u32, when: std::time::SystemTime }

struct LoginAudit;

impl DbAppend for LoginAudit {
    type Row = LoginAuditEntry;
    fn append(_: &DbConnection, _: &Self::Row) -> Result<()> { Ok(()) }
}

App::new()
    .add_plugins(MinimalPlugins)
    .add_plugins(DbPlugin)
    .init_db_journal::<LoginAudit>();
```

## Per-table connection override

Each table can opt into a separate database via `DbTableSettings::new(...)`
and `app.insert_db_table_settings::<MyTable>(settings)`. When the override is
absent, the table uses the shared `DbConnection` from `DbPlugin`.

## Type reference

| Type | Role |
|---|---|
| `Table` | Marker trait that turns a struct into a `Tables<T>` resource |
| `Tables<T>` | Resource holding the table value plus its dirty epoch |
| `Db<T>` | Read-only system parameter |
| `DbMut<T>` | Mutable system parameter that auto-bumps the dirty epoch |
| `DbConnection` | Shared connection resource opened by `DbPlugin` |
| `DbDriver` | Multi-backend Diesel connection (raw SQL) |
| `DbBackend` | Active backend label |
| `DbSettings` | Connection URL + SQLite pragmas |
| `DbTable` | Snapshot persistence trait |
| `DbAppend` | Append-only journal trait |
| `DbTableSettings<T>` | Per-table flush cadence and override |
| `AppDbExt` | `init_db_table` / `insert_db_table` |
| `AppDbPersistenceExt` | `init_db_persistent` / `insert_db_table_settings` / `init_db_journal` |

## SQL helpers

The driver exposes typed query builders (`PendingStatement`, `PendingInsert`)
and the `DbRecord` trait that `#[database_model(...)]` generates. Use the
`prelude::sql_query` re-export for raw SQL when needed.
