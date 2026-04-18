use bevy::prelude::*;
use mlua::{UserData, UserDataMethods};

use crate::{
    api::{json_to_lua, lua_to_json},
    runtime::ScriptRegistry,
    world_cell,
};

/// Lua UserData proxy for a Bevy entity.
///
/// ```lua
/// local entity = world:entity(id)
/// local hp = entity:get("Health")
/// entity:set("Health", { value = hp.value - 5 })
/// entity:trigger("TeleportIntent", { to = { x = 0, y = 0 } })
/// ```
pub struct EntityProxy {
    pub(crate) id: Entity,
}

impl UserData for EntityProxy {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("get", |lua, this, component_name: String| {
            // Copy the getter fn ptr out of the registry (immutable borrow of world).
            let get_fn = world_cell::with(|world| {
                world
                    .resource::<ScriptRegistry>()
                    .components
                    .get(&component_name)
                    .map(|accessor| accessor.get)
            });

            let Some(get_fn) = get_fn else {
                return Ok(mlua::Value::Nil);
            };

            // Call the getter (mutable world, no registry borrow).
            let json = world_cell::with(|world| get_fn(this.id, world));
            match json {
                Some(json_value) => json_to_lua(lua, json_value),
                None => Ok(mlua::Value::Nil),
            }
        });

        methods.add_method(
            "set",
            |_lua, this, (component_name, lua_value): (String, mlua::Value)| {
                let json = lua_to_json(lua_value)?;

                // Copy setter fn ptr.
                let set_fn = world_cell::with(|world| {
                    world
                        .resource::<ScriptRegistry>()
                        .components
                        .get(&component_name)
                        .map(|accessor| accessor.set)
                });

                if let Some(set_fn) = set_fn {
                    // Call setter.
                    world_cell::with(|world| set_fn(this.id, world, json));
                }

                Ok(())
            },
        );

        methods.add_method(
            "trigger",
            |_lua, this, (trigger_name, args_table): (String, mlua::Table)| {
                let json = lua_to_json(mlua::Value::Table(args_table))?;

                // Copy fire fn ptr.
                let fire_fn = world_cell::with(|world| {
                    world
                        .resource::<ScriptRegistry>()
                        .triggers
                        .get(&trigger_name)
                        .map(|trigger| trigger.fire)
                });

                if let Some(fire_fn) = fire_fn {
                    // Fire trigger.
                    world_cell::with(|world| fire_fn(this.id, world, json));
                }

                Ok(())
            },
        );

        // Returns the raw entity bits as a Lua integer.
        methods.add_method("id", |_lua, this, ()| Ok(this.id.to_bits() as i64));
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::{ComponentAccessor, LuaRuntime, ScriptRegistry, TriggerAccessor};
    use bevy::prelude::*;

    #[derive(Component)]
    struct TestHealth {
        value: i32,
    }

    #[derive(Resource, Default)]
    struct TriggerFired(bool);

    fn setup() -> (LuaRuntime, World) {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.init_resource::<TriggerFired>();

        world.resource_mut::<ScriptRegistry>().register_component(
            "TestHealth",
            ComponentAccessor {
                get: |entity, world| {
                    world
                        .get::<TestHealth>(entity)
                        .map(|health| serde_json::json!({ "value": health.value }))
                },
                set: |entity, world, json| {
                    if let Some(value) = json.get("value").and_then(|v| v.as_i64()) {
                        world.entity_mut(entity).insert(TestHealth {
                            value: value as i32,
                        });
                    }
                },
                component_id: |world| world.register_component::<TestHealth>(),
            },
        );

        world.resource_mut::<ScriptRegistry>().register_trigger(
            "Heal",
            TriggerAccessor {
                fire: |_entity, world, _json| {
                    world.resource_mut::<TriggerFired>().0 = true;
                },
            },
        );

        (LuaRuntime::new(), world)
    }

    fn run(runtime: &LuaRuntime, world: &mut World, lua: &str) {
        runtime
            .scope(world)
            .exec(lua)
            .expect("lua exec should succeed");
    }

    #[test]
    fn get_returns_component_as_table() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 42 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            local health = entity:get('TestHealth')
            assert(health ~= nil)
            assert(health.value == 42, 'expected 42, got ' .. tostring(health.value))
        ",
                entity.to_bits()
            ),
        );
    }

    #[test]
    fn get_returns_nil_for_unregistered_component_name() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            assert(entity:get('Nonexistent') == nil)
        ",
                entity.to_bits()
            ),
        );
    }

    #[test]
    fn get_returns_nil_when_entity_lacks_the_component() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id(); // no TestHealth

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            assert(entity:get('TestHealth') == nil)
        ",
                entity.to_bits()
            ),
        );
    }

    #[test]
    fn set_inserts_or_updates_component() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 0 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:set('TestHealth', {{ value = 99 }})
        ",
                entity.to_bits()
            ),
        );

        assert_eq!(
            world
                .get::<TestHealth>(entity)
                .expect("TestHealth should be present")
                .value,
            99
        );
    }

    #[test]
    fn set_is_noop_for_unregistered_component_name() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:set('Nonexistent', {{ value = 1 }})
        ",
                entity.to_bits()
            ),
        );
        // must not panic
    }

    #[test]
    fn trigger_fires_registered_trigger() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:trigger('Heal', {{}})
        ",
                entity.to_bits()
            ),
        );

        assert!(world.resource::<TriggerFired>().0);
    }

    #[test]
    fn trigger_is_noop_for_unregistered_trigger_name() {
        let (runtime, mut world) = setup();
        let entity = world.spawn_empty().id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:trigger('Nonexistent', {{}})
        ",
                entity.to_bits()
            ),
        );
        // must not panic
    }

    #[test]
    fn id_returns_entity_bits_as_integer() {
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
    fn get_then_modify_then_set_round_trip() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 10 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            local health = entity:get('TestHealth')
            entity:set('TestHealth', {{ value = health.value + 5 }})
        ",
                entity.to_bits()
            ),
        );

        assert_eq!(
            world
                .get::<TestHealth>(entity)
                .expect("TestHealth should be present")
                .value,
            15
        );
    }

    #[test]
    fn trigger_passes_args_json_to_handler() {
        #[derive(Resource, Default)]
        struct LastAmount(i64);

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

        let runtime = LuaRuntime::new();
        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .exec(&format!(
                "
            local entity = world:entity({})
            entity:trigger('Heal', {{ amount = 25 }})
        ",
                entity.to_bits()
            ))
            .expect("lua exec should succeed");

        assert_eq!(world.resource::<LastAmount>().0, 25);
    }

    #[test]
    fn set_with_unrecognised_field_shape_does_not_panic() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 0 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            entity:set('TestHealth', {{ wrong_field = 'oops' }})
        ",
                entity.to_bits()
            ),
        );

        assert_eq!(
            world
                .get::<TestHealth>(entity)
                .expect("TestHealth should be present")
                .value,
            0
        );
    }

    #[test]
    fn get_is_case_sensitive_for_component_name() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 1 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({})
            assert(entity:get('testhealth') == nil, 'wrong-case name should return nil')
            assert(entity:get('TESTHEALTH') == nil, 'uppercase name should return nil')
            assert(entity:get('TestHealth') ~= nil, 'exact-case name should return the component')
        ",
                entity.to_bits()
            ),
        );
    }

    #[test]
    fn get_returns_nil_when_entity_is_dead() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 5 }).id();
        let bits = entity.to_bits() as i64;
        world.despawn(entity);

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local entity = world:entity({bits})
            assert(entity:get('TestHealth') == nil, 'dead entity should return nil')
        "
            ),
        );
    }

    #[test]
    fn id_matches_entity_bits_after_spawn() {
        let (runtime, mut world) = setup();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            assert(world:entity({a}):id() == {a})
            assert(world:entity({b}):id() == {b})
            assert(world:entity({a}):id() ~= world:entity({b}):id())
        ",
                a = a.to_bits() as i64,
                b = b.to_bits() as i64
            ),
        );
    }
}
