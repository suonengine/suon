# suon_macros

Procedural macros shared across the Suon MMORPG workspace.

`suon_macros` provides four derive macros and one attribute macro:

- `#[derive(Table)]` implements the `Table` trait for typed table structs
- `#[derive(LuaComponent)]` derives `Component` and registers a Lua-accessible component
- `#[derive(LuaHook)]` derives a typed Lua hook payload
- `#[derive(DocumentedToml)]` generates documented TOML serialization
- `#[database_model(table = "...")]` generates Diesel schema, query, and insert glue

## Installation

```toml
[dependencies]
suon_macros = { path = "../suon_macros" }
```

## `#[derive(Table)]`

Marks a struct as a `suon_database` table. Pair with
`suon_database::prelude::AppDbExt` to register it at startup.

```rust,ignore
use suon_macros::Table;

#[derive(Table, Default)]
struct MonsterTable {
    rows: Vec<Monster>,
}
```

The macro implements `Table` with `Self: Send + Sync + 'static` bounds. The
struct can then be wrapped in a `Tables<MonsterTable>` resource and accessed
through `Db<MonsterTable>` / `DbMut<MonsterTable>`.

## `#[database_model(table = "...")]`

Generates Diesel `table!`, `Queryable`, `Selectable`, `Insertable`, and
backend-specific `CREATE TABLE IF NOT EXISTS` helpers from a Rust struct, plus
the `suon_database::prelude::DbRecord` impl.

```rust,ignore
use suon_macros::database_model;

#[database_model(table = "actors")]
#[derive(Debug, Clone)]
struct ActorRecord {
    #[database(primary_key)]
    id: i64,
    name: String,
}
```

Field attributes:

- `#[database(primary_key)]` marks one or more columns as the primary key
- `#[database(auto)]` adds backend-appropriate auto-increment to a primary key
- `#[database(column_name = "name")]` overrides the SQL column name

The generated impl block exposes:

- `Record::query()` — `PendingStatement<'_, _, Record>` for select queries
- `Record::ensure_table(driver, backend)` — runs `CREATE TABLE IF NOT EXISTS`
- `Record::create_table_sql(backend)` — returns the backend-specific DDL string

## `#[derive(LuaComponent)]`

Derives both `Component` and the `LuaComponent` trait needed by `suon_lua`.
The component is automatically registered with the `ScriptRegistry` the first
time it is inserted into any entity.

**Do not** add `#[derive(Component)]` alongside this macro because it is
already derived.

```rust,ignore
use serde::{Deserialize, Serialize};
use suon_macros::LuaComponent;

#[derive(LuaComponent, Serialize, Deserialize)]
struct Mana {
    current: i32,
    max: i32,
}
```

The Lua global name defaults to the struct name. Override it with
`#[lua(name = "...")]`:

```rust,ignore
#[derive(LuaComponent, Serialize, Deserialize)]
#[lua(name = "MP")]
struct Mana {
    current: i32,
    max: i32,
}
```

## `#[derive(LuaHook)]`

Derives the `Hook` trait for a struct used as a typed hook payload. The
default Lua method name is `on{StructName}`. Override it with
`#[lua(name = "...")]`.

```rust,ignore
use serde::Serialize;
use suon_macros::LuaHook;

#[derive(Serialize, LuaHook)]
struct Damage {
    amount: i32,
}

#[derive(Serialize, LuaHook)]
#[lua(name = "onTick")]
struct Tick;
```

Struct fields are serialized into positional Lua arguments. A unit struct
produces a zero-argument hook.
