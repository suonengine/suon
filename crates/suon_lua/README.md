# suon_lua

Bridge between Bevy ECS and Lua 5.4 via `mlua`.

`suon_lua` lets you:

- attach Lua scripts to entities
- call Lua hooks from Bevy systems
- read and write ECS components from Lua
- iterate entities with `Query(...)`
- fire Rust-registered triggers from Lua

The current Lua API is centered around:

- `Entity(id)` for direct entity access
- `Query("A", "B", ...)` for ECS iteration

The Lua surface is intentionally small: use `Entity(...)` and `Query(...)`.

## Installation

Add the plugin and register the components you want to expose to Lua.

`suon_lua` depends on `mlua` with vendored Lua enabled, so the crate embeds the
Lua VM and does not require a separate Lua installation.

Example dependencies:

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
use suon_macros::LuaComponent;

#[derive(Component, LuaComponent, Serialize, Deserialize, Clone)]
struct Health {
    value: i32,
}

fn spawn_actor(mut commands: Commands) {
    commands.spawn((
        Health { value: 100 },
        LuaScript::new(
            r#"
            function Entity:onTick()
                local health = self:get("Health")
                if health ~= nil then
                    self:set("Health", { value = health.value - 1 })
                end
            end
            "#,
        ),
    ));
}

fn drive_scripts(mut commands: Commands, query: Query<Entity, With<LuaScript>>) {
    for entity in &query {
        commands.lua_hook(entity, "onTick");
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
- `LuaCommands::lua_hook` schedules a hook call on an entity.
- `LuaCommands::lua_execute` schedules execution of an arbitrary Lua snippet.
- `ScriptRegistry` maps Lua-visible names like `"Health"` to typed Rust accessors.

Components derived with `#[derive(LuaComponent)]` can be registered
automatically the first time they appear in the world, or manually through
`AppLuaExt::register_lua_component::<T>()`.

## Lua API

### `Entity(id)`

Returns an `EntityProxy` for the entity.

Example:

```lua
local entity = Entity(123)
local health = entity:get("Health")
if health ~= nil then
    entity:set("Health", { value = health.value - 10 })
end
```

The `Entity` global is also the table where hook methods are defined:

```lua
function Entity:onHit()
    local health = self:get("Health")
    if health ~= nil then
        self:set("Health", { value = health.value - 10 })
    end
end
```

Available proxy methods:

- `entity:get("Name")`
- `entity:set("Name", { ... })`
- `entity:trigger("Name", { ... })`
- `entity:id()`

### `Query("A", "B", ...)`

Returns a `QueryProxy` with an `:iter()` method.

Example:

```lua
for id, health, position in Query("Health", "Position"):iter() do
    if health.value <= 0 then
        position.x = 0
        position.y = 0
    end
end
```

The iterator yields:

- the entity id first
- then one value per requested component, in order

## Running Lua From Rust

### Calling a hook

```rust,ignore
fn tick_scripts(mut commands: Commands, query: Query<Entity, With<LuaScript>>) {
    for entity in &query {
        commands.lua_hook(entity, "onTick");
    }
}
```

### Executing an ad-hoc snippet

```rust,ignore
fn heal_low_health(mut commands: Commands) {
    commands.lua_execute(
        r#"
        for id, health in Query("Health"):iter() do
            if health.value < 10 then
                Entity(id):set("Health", { value = 100 })
            end
        end
        "#,
    );
}
```

## Component Registration

All Lua component access goes through names registered in `ScriptRegistry`.

If a component implements `LuaComponent`, its accessor needs to:

- serialize the component into Lua in `get`
- deserialize a Lua value back into the component in `set`
- expose a stable Lua-visible name with `lua_name()`

In practice, the simplest path is to use `#[derive(LuaComponent)]`.

## Important Semantics

- `entity:get("Name")` returns `nil` if the component does not exist or is not registered.
- `entity:set("Name", value)` is a no-op if the component is not registered.
- `Query(...)` only returns entities that contain all requested components.
- If any component name in the query is not registered, the iteration is empty.
- Writes made through component proxies returned by `Query(...):iter()` are batched and applied on the next iteration step.
- Lua hooks are method-only and should be defined as `Entity:onEvent()`.

## API Status

`Entity(...)` and `Query(...)` are the public Lua entry points and are covered by tests.
