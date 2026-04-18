use mlua::{Lua, UserData, UserDataMethods};

use crate::api::{entity::EntityProxy, query::QueryProxy};

/// Lua UserData for the `world` global, injected at the start of every hook.
///
/// ```lua
/// local entity = world:entity(id)
/// for id, hp in world:query("Health"):iter() do ... end
/// ```
struct WorldProxy;

impl UserData for WorldProxy {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("entity", |_lua, _, id: i64| {
            let entity = bevy::prelude::Entity::from_bits(id as u64);
            Ok(EntityProxy { id: entity })
        });

        methods.add_method("query", |_, _, components: mlua::Variadic<String>| {
            Ok(QueryProxy {
                components: components.into_iter().collect(),
            })
        });
    }
}

/// Registers Lua globals. Called once in [`LuaRuntime::new`].
///
/// Globals registered:
/// - `world` — [`WorldProxy`] for ECS access
/// - `Entity` — empty table; scripts add hook methods to it: `function Entity:onTeleport()`
pub(crate) fn register_world_api(lua: &Lua) -> mlua::Result<()> {
    let globals = lua.globals();
    globals.set("world", lua.create_userdata(WorldProxy)?)?;
    globals.set("Entity", lua.create_table()?)?;
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
            .expect("lua exec should succeed");
    }

    #[test]
    fn register_world_api_injects_world_global() {
        let lua = Lua::new();
        register_world_api(&lua).expect("world api registration should succeed");

        let world_val: mlua::Value = lua
            .globals()
            .get("world")
            .expect("world global should be set");

        assert!(!matches!(world_val, mlua::Value::Nil));
    }

    #[test]
    fn register_world_api_injects_entity_table_global() {
        let lua = Lua::new();
        register_world_api(&lua).expect("world api registration should succeed");

        let entity_val: mlua::Value = lua
            .globals()
            .get("Entity")
            .expect("Entity global should be set");

        assert!(matches!(entity_val, mlua::Value::Table(_)));
    }

    #[test]
    fn world_entity_returns_proxy_with_correct_id() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();
        let expected = entity.to_bits() as i64;

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({expected})
            assert(entity:id() == {expected})
        "
            ),
        );
    }

    #[test]
    fn world_query_with_no_match_iterates_zero_times() {
        let (runtime, mut world) = setup();

        run(
            &runtime,
            &mut world,
            "
            local count = 0
            for id in world:query('Nonexistent'):iter() do
                count = count + 1
            end
            assert(count == 0)
        ",
        );
    }

    #[test]
    fn world_query_variadic_accepts_multiple_component_names() {
        let (runtime, mut world) = setup();

        run(
            &runtime,
            &mut world,
            "
            local q = world:query('A', 'B', 'C')
            assert(q ~= nil)
        ",
        );
    }
}
