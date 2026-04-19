//! Registers the Lua-facing ECS globals.
//!
//! Called once from [`LuaRuntime::new`]. The runtime exposes `Entity(id)` for
//! entity access and `Query(...)` for ECS iteration.

use mlua::Lua;

use crate::api::{entity::EntityProxy, query::QueryProxy};

fn entity_proxy_from_id(id: i64) -> EntityProxy {
    let entity = bevy::prelude::Entity::from_bits(id as u64);
    EntityProxy { id: entity }
}

fn query_proxy_from_components(components: mlua::Variadic<String>) -> QueryProxy {
    QueryProxy {
        components: components.into_iter().collect(),
    }
}

/// Registers Lua globals. Called once in [`LuaRuntime::new`].
///
/// Globals registered:
/// - `Entity` as a callable table for both hooks and `Entity(id)` construction
/// - `Query` as a global constructor for `Query(...)`
pub(crate) fn register_world_api(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();

    let entity = lua.create_table()?;
    let entity_metatable = lua.create_table()?;
    entity_metatable.set(
        "__call",
        lua.create_function(|_lua, (_class, id): (mlua::Table, i64)| Ok(entity_proxy_from_id(id)))?,
    )?;
    let _ = entity.set_metatable(Some(entity_metatable));

    globals.set("Entity", entity)?;
    globals.set(
        "Query",
        lua.create_function(|_lua, components: mlua::Variadic<String>| {
            Ok(query_proxy_from_components(components))
        })?,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{LuaRuntime, ScriptRegistry};
    use bevy::prelude::*;

    fn setup() -> (LuaRuntime, World) {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        (LuaRuntime::new(), world)
    }

    fn run(runtime: &LuaRuntime, world: &mut World, lua: &str) {
        runtime
            .scope(world)
            .execute(lua)
            .unwrap_or_else(|error| panic!("lua exec should succeed: {error}"));
    }

    #[test]
    fn register_world_api_injects_entity_table_global() {
        let lua = Lua::new();
        register_world_api(&lua)
            .unwrap_or_else(|error| panic!("Lua globals registration should succeed: {error}"));

        let entity_val: mlua::Value = lua
            .globals()
            .get("Entity")
            .unwrap_or_else(|error| panic!("Entity global should be set: {error}"));

        assert!(matches!(entity_val, mlua::Value::Table(_)));
    }

    #[test]
    fn register_world_api_injects_query_function_global() {
        let lua = Lua::new();
        register_world_api(&lua)
            .unwrap_or_else(|error| panic!("Lua globals registration should succeed: {error}"));

        let query_val: mlua::Value = lua
            .globals()
            .get("Query")
            .unwrap_or_else(|error| panic!("Query global should be set: {error}"));

        assert!(matches!(query_val, mlua::Value::Function(_)));
    }

    #[test]
    fn entity_constructor_returns_proxy_with_correct_id() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();
        let expected = entity.to_bits() as i64;

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = Entity({expected})
            assert(entity:id() == {expected})
        "
            ),
        );
    }

    #[test]
    fn query_constructor_with_no_match_iterates_zero_times() {
        let (runtime, mut world) = setup();

        run(
            &runtime,
            &mut world,
            "
            local count = 0
            for id in Query('Nonexistent'):iter() do
                count = count + 1
            end
            assert(count == 0)
        ",
        );
    }

    #[test]
    fn query_constructor_variadic_accepts_multiple_component_names() {
        let (runtime, mut world) = setup();

        run(
            &runtime,
            &mut world,
            "
            local q = Query('A', 'B', 'C')
            assert(q ~= nil)
        ",
        );
    }
}
