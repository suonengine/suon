//! Registers the Lua-facing ECS globals.
//!
//! Called once from [`LuaRuntime::new`]. The runtime exposes `Entity(id)` for
//! entity access and `Query(...)` for ECS iteration.

use crate::api::{entity::EntityProxy, query::QueryProxy};

/// Extension methods for registering the Lua-facing ECS globals on [`mlua::Lua`].
pub(crate) trait LuaWorldApiExt {
    /// Registers the `Entity` and `Query` globals used by the Suon Lua API.
    fn register_world_api(&self) -> mlua::Result<()>;
}

impl LuaWorldApiExt for mlua::Lua {
    fn register_world_api(&self) -> mlua::Result<()> {
        let globals = self.globals();

        let entity_table = self.create_table()?;
        let entity_table_metatable = self.create_table()?;
        entity_table_metatable.set(
            "__call",
            self.create_function(|_lua, (_entity_class, entity_bits): (mlua::Table, i64)| {
                Ok(EntityProxy {
                    id: bevy::prelude::Entity::from_bits(entity_bits as u64),
                })
            })?,
        )?;

        entity_table.set_metatable(Some(entity_table_metatable))?;

        globals.set("Entity", entity_table)?;
        globals.set(
            "Query",
            self.create_function(|_lua, components: mlua::Variadic<mlua::Table>| {
                let mut component_names = Vec::new();
                for table in components {
                    if let Ok(name) = table.get::<String>("__component") {
                        component_names.push(name);
                    }
                }
                Ok(QueryProxy {
                    components: component_names,
                })
            })?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{LuaRuntime, ScriptRegistry};
    use bevy::prelude::*;

    #[test]
    fn register_world_api_injects_entity_table_global() {
        let lua = mlua::Lua::new();

        lua.register_world_api()
            .expect("Lua globals registration should succeed");

        let entity_val: mlua::Value = lua
            .globals()
            .get("Entity")
            .expect("Entity global should be set");

        assert!(matches!(entity_val, mlua::Value::Table(_)));
    }

    #[test]
    fn register_world_api_injects_query_function_global() {
        let lua = mlua::Lua::new();

        lua.register_world_api()
            .expect("Lua globals registration should succeed");

        let query_val: mlua::Value = lua
            .globals()
            .get("Query")
            .expect("Query global should be set");

        assert!(matches!(query_val, mlua::Value::Function(_)));
    }

    #[test]
    fn entity_constructor_returns_proxy_with_correct_id() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();
        let expected = entity.to_bits() as i64;

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({expected})
            assert(entity:id() == {expected})
        "
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn query_constructor_with_no_match_iterates_zero_times() {
        use serde::{Deserialize, Serialize};
        use suon_macros::LuaComponent;

        #[derive(LuaComponent, Serialize, Deserialize)]
        struct Empty;

        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        // Spawn then despawn to register the component global without leaving any entity.
        let entity = world.spawn(Empty).id();
        world.despawn(entity);

        runtime
            .scope(&mut world)
            .execute(
                "
            local count = 0
            for id in Query(Empty):iter() do
                count = count + 1
            end
            assert(count == 0)
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn query_constructor_variadic_accepts_multiple_component_names() {
        use crate::runtime::ComponentAccessor;
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        for name in ["A", "B", "C"] {
            world.resource_mut::<ScriptRegistry>().register_component(
                name,
                ComponentAccessor {
                    get: |_, _| None,
                    set: |_, _, _| {},
                    component_id: |_| unreachable!(),
                },
            );
        }

        runtime
            .scope(&mut world)
            .execute(
                "
            local q = Query(A, B, C)
            assert(q ~= nil)
        ",
            )
            .expect("lua exec should succeed");
    }
}
