# suon_macros

Procedural macros shared across the Suon MMORPG workspace.

`suon_macros` provides three derive macros:

- `#[derive(Table)]` implements the `Table` trait for ECS table structs
- `#[derive(LuaComponent)]` derives `Component` and registers a Lua-accessible component
- `#[derive(LuaHook)]` derives a typed Lua hook payload

## Installation

```toml
[dependencies]
suon_macros = { path = "../suon_macros" }
```

## `#[derive(Table)]`

Marks a struct as a `suon_database` table. Pair with `suon_database::AppTablesExt` to
register it at startup.

```rust,ignore
use suon_macros::Table;

#[derive(Table, Default)]
struct MonsterTable {
    rows: Vec<Monster>,
}
```

## `#[derive(LuaComponent)]`

Derives both `Component` and the `LuaComponent` trait needed by `suon_lua`. The component
is automatically registered with the `ScriptRegistry` the first time it is inserted into
any entity.

**Do not** add `#[derive(Component)]` alongside this macro because it is already derived.

```rust,ignore
use serde::{Deserialize, Serialize};
use suon_macros::LuaComponent;

#[derive(LuaComponent, Serialize, Deserialize)]
struct Mana {
    current: i32,
    max: i32,
}
```

The Lua global name defaults to the struct name. Override it with `#[lua(name = "...")]`:

```rust,ignore
#[derive(LuaComponent, Serialize, Deserialize)]
#[lua(name = "MP")]
struct Mana {
    current: i32,
    max: i32,
}
```

## `#[derive(LuaHook)]`

Derives the `Hook` trait for a struct used as a typed hook payload. The default Lua method
name is `on{StructName}`. Override it with `#[lua(name = "...")]`.

```rust,ignore
use serde::Serialize;
use suon_macros::LuaHook;

// Invokes Entity:onDamage(amount) in Lua.
#[derive(Serialize, LuaHook)]
struct Damage {
    amount: i32,
}

// Invokes Entity:onTick() in Lua.
#[derive(Serialize, LuaHook)]
#[lua(name = "onTick")]
struct Tick;
```

Struct fields are serialized into positional Lua arguments. A unit struct produces a
zero-argument hook.
