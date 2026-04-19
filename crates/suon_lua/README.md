# suon_lua

Bridge between Bevy ECS and Lua 5.4 via `mlua`.

`suon_lua` lets you:

- attach Lua scripts to entities
- call Lua hooks from Bevy systems
- read and write ECS components from Lua
- iterate entities with `Query(...)`
- fire Rust-registered triggers from Lua

## Installation

`suon_lua` depends on `mlua` with vendored Lua enabled — the VM is embedded and
requires no separate Lua installation.

```toml
[dependencies]
bevy = "..."
serde = { version = "...", features = ["derive"] }
suon_lua = { path = "../suon_lua" }
suon_macros = { path = "../suon_macros" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use suon_lua::{LuaCommands, LuaPlugin, LuaScript};
use suon_macros::{LuaComponent, LuaHook};

// LuaComponent already derives Component — do not add #[derive(Component)].
#[derive(LuaComponent, Serialize, Deserialize)]
struct Health {
    value: i32,
}

#[derive(Serialize, LuaHook)]
#[lua(name = "onTick")]
struct Tick;

fn spawn_actor(mut commands: Commands) {
    commands.spawn((
        Health { value: 100 },
        LuaScript::new(
            "function Entity:onTick()
                local hp = self:get(Health)
                if hp ~= nil then
                    hp.value = hp.value - 1
                end
            end",
        ),
    ));
}

fn drive_scripts(mut commands: Commands, query: Query<Entity, With<LuaScript>>) {
    for entity in &query {
        assert!(commands.lua_hook(entity, Tick).is_ok());
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, LuaPlugin))
        .add_systems(Startup, spawn_actor)
        .add_systems(Update, drive_scripts)
        .run();
}
```

## How It Works

- `LuaPlugin` inserts `LuaRuntime` as a non-send resource and initializes `ScriptRegistry`.
- `LuaScript` stores Lua source attached to an entity.
- `LuaCommands::lua_hook` schedules a typed hook call on an entity.
- `LuaCommands::lua_execute` schedules execution of an arbitrary Lua snippet.
- `ScriptRegistry` maps component type globals to typed Rust accessors.

Components derived with `#[derive(LuaComponent)]` register automatically the
first time they appear in the world, or manually via
`AppLuaExt::register_lua_component::<T>()`.

## Lua API

### `Entity(id)`

Returns an `EntityProxy` for the entity. The same global is where hook methods
are defined.

```lua
local entity = Entity(123)
local hp = entity:get(Health)
if hp ~= nil then
    hp.value = hp.value - 10   -- written back to ECS immediately
end
```

Available proxy methods:

- `entity:get(ComponentType)` — returns a mutable proxy or `nil`
- `entity:trigger("Name", { ... })` — fires a registered Rust trigger
- `entity:id()` — returns the entity's raw bit representation

Mutations are applied by assigning to fields on the proxy:

```lua
function Entity:onHit()
    local hp = self:get(Health)
    if hp ~= nil then
        hp.value = hp.value - 10
    end
end
```

### `Query(A, B, ...)`

Returns a `QueryProxy` with an `:iter()` method that yields `id, a, b, ...`
for every entity that has all the requested components.

```lua
for id, health, position in Query(Health, Position):iter() do
    if health.value <= 0 then
        position.x = 0
        position.y = 0
    end
end
```

Component variables yielded by the iterator are mutable proxies — assigning to
their fields batches a write that is applied before the next iteration step.

## Running Lua From Rust

### Calling a hook

```rust,ignore
fn tick_scripts(mut commands: Commands, query: Query<Entity, With<LuaScript>>) {
    for entity in &query {
        assert!(commands.lua_hook(entity, Tick).is_ok());
    }
}
```

### Calling a hook with arguments

```rust,ignore
#[derive(Serialize, LuaHook)]
struct Move {
    from: (i32, i32),
    to: (i32, i32),
}

fn move_entity(mut commands: Commands, entity: Entity) {
    commands
        .lua_hook(entity, Move { from: (0, 0), to: (10, 20) })
        .is_ok();
}
```

Matches this Lua hook:

```lua
function Entity:onMove(from, to)
    local pos = self:get(Position)
    if pos ~= nil then
        pos.x = to[1]
        pos.y = to[2]
    end
end
```

### Executing an ad-hoc snippet

```rust,ignore
fn heal_low_health(mut commands: Commands) {
    commands.lua_execute(
        "for id, hp in Query(Health):iter() do
            if hp.value < 10 then
                hp.value = 100
            end
        end",
    );
}
```

## Component Registration

All component access goes through `ScriptRegistry`. `#[derive(LuaComponent)]`
handles registration automatically — it generates the `Component` impl (do not
add `#[derive(Component)]` alongside it) and registers a get/set accessor the
first time the component is inserted into any entity.

## Semantics

- `entity:get(C)` returns `nil` if the entity lacks the component or `C` is not registered.
- Assigning to a proxy field calls the component's setter immediately.
- `Query(...)` only yields entities that have **all** requested components. An
  unregistered component makes the whole query empty.
- Query proxy writes are batched and flushed before each iteration step.
- Hook functions must be defined as `Entity:onEvent()` — `self` is the entity proxy.
- Hook struct fields are serialized into positional Lua arguments.
