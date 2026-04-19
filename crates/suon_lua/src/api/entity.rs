//! [`EntityProxy`] — the Lua userdata object returned by `Entity(id)`.
//!
//! Exposes `get`, `trigger`, and `id` methods to Lua scripts.
//! All ECS access goes through [`world_cell::with`] so the proxy never holds a
//! borrow across a Lua call boundary.

use bevy::prelude::*;
use mlua::{UserData, UserDataMethods};
use std::rc::Rc;

use crate::{
    api::{IntoJsonValueExt, query::LuaQueryExt},
    runtime::ScriptRegistry,
    world_cell,
};

/// Lua UserData proxy for a Bevy entity.
///
/// ```lua
/// local entity = Entity(id)
/// local hp = entity:get(Health)
/// if hp ~= nil then
///     hp.value = hp.value - 5   -- written back to ECS immediately
/// end
/// entity:trigger("TeleportIntent", { to = { x = 0, y = 0 } })
/// ```
pub struct EntityProxy {
    /// Raw Bevy entity id represented by this Lua userdata instance.
    pub(crate) id: Entity,
}

impl UserData for EntityProxy {
    /// Registers the Lua-facing methods available on `Entity(id)` proxies.
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, entity_proxy, component: mlua::Table| {
            let Some(component_name) = component.get::<Option<String>>("__component")? else {
                return Ok(mlua::Value::Nil);
            };

            let accessor = world_cell::with(|world| {
                world
                    .resource::<ScriptRegistry>()
                    .components
                    .get(&component_name)
                    .map(|a| (a.get, a.set))
            });

            let Some((component_getter, component_setter)) = accessor else {
                return Ok(mlua::Value::Nil);
            };

            let component_snapshot =
                world_cell::with(|world| component_getter(entity_proxy.id, world));

            let Some(component_json) = component_snapshot else {
                return Ok(mlua::Value::Nil);
            };

            let inner_table = Rc::new(lua.create_lua_component_table(component_json)?);
            let entity_id = entity_proxy.id;

            let proxy_table = lua.create_table()?;
            let metatable = lua.create_table()?;

            let index_table = inner_table.clone();
            metatable.set(
                "__index",
                lua.create_function(move |_lua, (_proxy, key): (mlua::Table, String)| {
                    index_table.raw_get::<mlua::Value>(key)
                })?,
            )?;

            let newindex_table = inner_table.clone();
            metatable.set(
                "__newindex",
                lua.create_function(
                    move |_lua, (_proxy, key, value): (mlua::Table, String, mlua::Value)| {
                        newindex_table.raw_set(key, value)?;
                        let json =
                            mlua::Value::Table((*newindex_table).clone()).into_json_value()?;
                        world_cell::with(|world| component_setter(entity_id, world, json));
                        Ok(())
                    },
                )?,
            )?;

            proxy_table.set_metatable(Some(metatable))?;
            Ok(mlua::Value::Table(proxy_table))
        });

        methods.add_method(
            "trigger",
            |_lua, entity_proxy, (trigger_name, lua_args_table): (String, mlua::Table)| {
                let trigger_args_json = mlua::Value::Table(lua_args_table).into_json_value()?;

                let trigger_handler = world_cell::with(|world| {
                    world
                        .resource::<ScriptRegistry>()
                        .triggers
                        .get(&trigger_name)
                        .map(|trigger| trigger.fire)
                });

                if let Some(trigger_handler) = trigger_handler {
                    world_cell::with(|world| {
                        trigger_handler(entity_proxy.id, world, trigger_args_json)
                    });
                }

                Ok(())
            },
        );

        methods.add_method("id", |_lua, entity_proxy, ()| {
            Ok(entity_proxy.id.to_bits() as i64)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{LuaRuntime, ScriptRegistry, TriggerAccessor};
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};
    use suon_macros::LuaComponent;

    #[derive(LuaComponent, Serialize, Deserialize)]
    struct Health {
        value: i32,
    }

    #[derive(Resource, Default)]
    struct TriggerFired(bool);

    #[test]
    fn get_returns_component_as_table() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 42 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            local health = entity:get(Health)
            assert(health ~= nil)
            assert(health.value == 42, 'expected 42, got ' .. tostring(health.value))
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn get_proxy_assignment_writes_component_back_to_ecs() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 10 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            local health = entity:get(Health)
            health.value = health.value - 3
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(
            world
                .get::<Health>(entity)
                .expect("Health should be present")
                .value,
            7
        );
    }

    #[test]
    fn get_returns_nil_for_table_without_component_key() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            assert(entity:get({{}}) == nil)
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn get_returns_nil_when_entity_lacks_the_component() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        // Register Health global without spawning an entity that has it.
        let entity = world.spawn(Health { value: 0 }).id();
        world.despawn(entity);

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            assert(entity:get(Health) == nil)
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn trigger_fires_registered_trigger() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.init_resource::<TriggerFired>();

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Heal",
            TriggerAccessor {
                fire: |_entity, world, _json| {
                    world.resource_mut::<TriggerFired>().0 = true;
                },
            },
        );

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            entity:trigger('Heal', {{}})
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert!(world.resource::<TriggerFired>().0);
    }

    #[test]
    fn trigger_is_noop_for_unregistered_trigger_name() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            entity:trigger('Nonexistent', {{}})
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn trigger_returns_error_when_argument_is_not_a_table() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.init_resource::<TriggerFired>();

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Heal",
            TriggerAccessor {
                fire: |_entity, world, _json| {
                    world.resource_mut::<TriggerFired>().0 = true;
                },
            },
        );

        let entity = world.spawn_empty().id();

        let error = runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            entity:trigger('Heal', 123)
        ",
                entity.to_bits()
            ))
            .expect_err("non-table trigger arguments should error");

        assert!(
            error.to_string().to_lowercase().contains("table"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn id_returns_entity_bits_as_integer() {
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
    fn get_proxy_mutation_round_trip() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 10 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            local health = entity:get(Health)
            health.value = health.value + 5
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(
            world
                .get::<Health>(entity)
                .expect("Health should be present")
                .value,
            15
        );
    }

    #[test]
    fn trigger_passes_args_json_to_handler() {
        #[derive(Resource, Default)]
        struct LastAmount(i64);

        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.init_resource::<LastAmount>();

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Heal",
            TriggerAccessor {
                fire: |_entity, world, json| {
                    let amount = json.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
                    world.resource_mut::<LastAmount>().0 = amount;
                },
            },
        );

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            entity:trigger('Heal', {{ amount = 25 }})
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(world.resource::<LastAmount>().0, 25);
    }

    #[test]
    fn get_proxy_write_with_unrecognised_field_does_not_panic() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 0 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            local health = entity:get(Health)
            health.wrong_field = 'oops'
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(
            world
                .get::<Health>(entity)
                .expect("Health should be present")
                .value,
            0
        );
    }

    #[test]
    fn get_returns_component_for_correct_component_table() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 1 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({})
            assert(entity:get(Health) ~= nil, 'registered component table should return the \
                 component')
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn get_returns_nil_when_entity_is_dead() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 5 }).id();
        let bits = entity.to_bits() as i64;

        world.despawn(entity);

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local entity = Entity({bits})
            assert(entity:get(Health) == nil, 'dead entity should return nil')
        "
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn trigger_removes_component_from_entity() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "RemoveHealth",
            TriggerAccessor {
                fire: |entity, world, _json| {
                    world.entity_mut(entity).remove::<Health>();
                },
            },
        );

        let entity = world.spawn(Health { value: 100 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "Entity({}):trigger('RemoveHealth', {{}})",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert!(
            world.get::<Health>(entity).is_none(),
            "Health should have been removed by the trigger"
        );
    }

    #[test]
    fn trigger_chain_fires_both_handlers() {
        #[derive(Resource, Default)]
        struct FireLog(Vec<&'static str>);

        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.init_resource::<FireLog>();

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Inner",
            TriggerAccessor {
                fire: |_entity, world, _json| {
                    world.resource_mut::<FireLog>().0.push("inner");
                },
            },
        );

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Outer",
            TriggerAccessor {
                fire: |entity, world, _json| {
                    world.resource_mut::<FireLog>().0.push("outer");
                    // A trigger firing another trigger tests re-entrant ScriptRegistry access.
                    let inner_fire = world
                        .resource::<ScriptRegistry>()
                        .triggers
                        .get("Inner")
                        .map(|t| t.fire);
                    if let Some(fire) = inner_fire {
                        fire(entity, world, serde_json::Value::Null);
                    }
                },
            },
        );

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "Entity({}):trigger('Outer', {{}})",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(
            world.resource::<FireLog>().0.as_slice(),
            &["outer", "inner"],
            "outer trigger should fire first, then the inner trigger it chains to"
        );
    }

    #[test]
    fn id_matches_entity_bits_after_spawn() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            assert(Entity({a}):id() == {a})
            assert(Entity({b}):id() == {b})
            assert(Entity({a}):id() ~= Entity({b}):id())
        ",
                a = a.to_bits() as i64,
                b = b.to_bits() as i64
            ))
            .expect("lua exec should succeed");
    }
}
