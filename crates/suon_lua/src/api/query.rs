use bevy::{
    ecs::{component::ComponentId, query::QueryBuilder},
    prelude::*,
};
use mlua::{UserData, UserDataMethods};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    api::{json_to_lua, lua_to_json},
    runtime::ScriptRegistry,
    world_cell,
};

type ComponentEntry = (
    fn(&mut World) -> ComponentId,
    fn(Entity, &mut World) -> Option<serde_json::Value>,
    fn(Entity, &mut World, serde_json::Value),
);

type ComponentData = (serde_json::Value, fn(Entity, &mut World, serde_json::Value));

type PendingEntry = (
    Rc<RefCell<serde_json::Value>>,
    Rc<Cell<bool>>,
    Entity,
    fn(Entity, &mut World, serde_json::Value),
);

/// Lua UserData returned by `world:query(...)`.
///
/// ```lua
/// for id, hp in world:query("Health"):iter() do
///     hp.value = hp.value + 10   -- batched: written to ECS once per entity at end of each step
/// end
///
/// for id, hp, pos in world:query("Health", "Position"):iter() do
///     hp.value = 0   -- both fields written in a single component_set at end of step
///     pos.x = 0
/// end
/// ```
pub struct QueryProxy {
    pub(crate) components: Vec<String>,
}

impl UserData for QueryProxy {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("iter", |lua, this, ()| {
            let component_names = this.components.clone();

            let rows: Vec<(u64, Vec<ComponentData>)> =
                world_cell::with(|world| collect_query(world, &component_names));

            let rows = Rc::new(rows);
            let cursor = Rc::new(Cell::new(0usize));
            let pending: Rc<RefCell<Vec<PendingEntry>>> = Rc::new(RefCell::new(Vec::new()));

            let iter_fn = lua.create_function(move |lua, _: mlua::MultiValue| {
                // Flush dirty proxies from the previous iteration before advancing.
                for (data, dirty, entity, set_fn) in pending.borrow().iter() {
                    if dirty.get() {
                        dirty.set(false);
                        let snapshot = data.borrow().clone();
                        world_cell::with(|world| set_fn(*entity, world, snapshot));
                    }
                }
                pending.borrow_mut().clear();

                let index = cursor.get();
                if index >= rows.len() {
                    return Ok(mlua::MultiValue::new());
                }
                cursor.set(index + 1);

                let (entity_bits, ref components) = rows[index];
                let entity = Entity::from_bits(entity_bits);

                let mut multi = mlua::MultiValue::new();
                multi.push_back(mlua::Value::Integer(entity_bits as i64));

                for (component_json, set_fn) in components.iter() {
                    let data = Rc::new(RefCell::new(component_json.clone()));
                    let dirty = Rc::new(Cell::new(false));
                    let set_fn = *set_fn;

                    pending
                        .borrow_mut()
                        .push((data.clone(), dirty.clone(), entity, set_fn));

                    let proxy = lua.create_table()?;
                    let meta = lua.create_table()?;

                    let data_idx = data.clone();
                    meta.set(
                        "__index",
                        lua.create_function(move |lua, (_proxy, key): (mlua::Table, String)| {
                            let val = data_idx
                                .borrow()
                                .get(&key)
                                .cloned()
                                .unwrap_or(serde_json::Value::Null);
                            json_to_lua(lua, val)
                        })?,
                    )?;

                    let data_ni = data.clone();
                    let dirty_ni = dirty.clone();
                    meta.set(
                        "__newindex",
                        lua.create_function(
                            move |_lua, (_proxy, key, lua_val): (mlua::Table, String, mlua::Value)| {
                                let mut d = data_ni.borrow_mut();
                                if let serde_json::Value::Object(ref mut map) = *d {
                                    map.insert(key, lua_to_json(lua_val)?);
                                }
                                dirty_ni.set(true);
                                Ok(())
                            },
                        )?,
                    )?;

                    let _ = proxy.set_metatable(Some(meta));
                    multi.push_back(mlua::Value::Table(proxy));
                }

                Ok(multi)
            })?;

            Ok(iter_fn)
        });
    }
}

/// Runs the query against the world and collects results.
///
/// Uses [`resource_scope`] so the registry and world can be borrowed independently,
/// then builds a dynamic [`QueryState`] via [`QueryBuilder`].
fn collect_query(world: &mut World, component_names: &[String]) -> Vec<(u64, Vec<ComponentData>)> {
    let mut results = Vec::new();

    world.resource_scope(|world, registry: Mut<ScriptRegistry>| {
        let entries: Vec<ComponentEntry> = component_names
            .iter()
            .filter_map(|name| {
                registry
                    .components
                    .get(name.as_str())
                    .map(|accessor| (accessor.component_id, accessor.get, accessor.set))
            })
            .collect();

        let component_ids: Vec<ComponentId> = entries
            .iter()
            .map(|(init_id, _, _)| init_id(world))
            .collect();

        let get_fns: Vec<fn(Entity, &mut World) -> Option<serde_json::Value>> =
            entries.iter().map(|(_, get, _)| *get).collect();

        let set_fns: Vec<fn(Entity, &mut World, serde_json::Value)> =
            entries.iter().map(|(_, _, set)| *set).collect();

        if component_ids.is_empty() {
            return;
        }

        let mut builder = QueryBuilder::<Entity>::new(world);
        for &component_id in &component_ids {
            builder.with_id(component_id);
        }

        let mut query_state = builder.build();
        let entities: Vec<Entity> = query_state.iter(world).collect();

        for entity in entities {
            let components: Vec<ComponentData> = get_fns
                .iter()
                .zip(set_fns.iter())
                .filter_map(|(get, set)| get(entity, world).map(|json| (json, *set)))
                .collect();
            results.push((entity.to_bits(), components));
        }
    });

    results
}

#[cfg(test)]
mod tests {
    use crate::runtime::{ComponentAccessor, LuaRuntime, ScriptRegistry};
    use bevy::prelude::*;

    #[derive(Component)]
    struct TestHealth {
        value: i32,
    }

    #[derive(Component)]
    struct TestPosition {
        x: i32,
        y: i32,
    }

    fn setup() -> (LuaRuntime, World) {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        {
            let mut registry = world.resource_mut::<ScriptRegistry>();

            registry.register_component(
                "TestHealth",
                ComponentAccessor {
                    get: |entity, world| {
                        world
                            .get::<TestHealth>(entity)
                            .map(|health| serde_json::json!({ "value": health.value }))
                    },
                    set: |_, _, _| {},
                    component_id: |world| world.register_component::<TestHealth>(),
                },
            );

            registry.register_component(
                "TestPosition",
                ComponentAccessor {
                    get: |entity, world| {
                        world
                            .get::<TestPosition>(entity)
                            .map(|pos| serde_json::json!({ "x": pos.x, "y": pos.y }))
                    },
                    set: |_, _, _| {},
                    component_id: |world| world.register_component::<TestPosition>(),
                },
            );
        }

        (LuaRuntime::new(), world)
    }

    fn writable_health_accessor() -> ComponentAccessor {
        ComponentAccessor {
            get: |entity, world| {
                world
                    .get::<TestHealth>(entity)
                    .map(|h| serde_json::json!({ "value": h.value }))
            },
            set: |entity, world, json| {
                if let Some(v) = json.get("value").and_then(|v| v.as_i64()) {
                    world
                        .entity_mut(entity)
                        .insert(TestHealth { value: v as i32 });
                }
            },
            component_id: |world| world.register_component::<TestHealth>(),
        }
    }

    fn run(runtime: &LuaRuntime, world: &mut World, lua: &str) {
        runtime
            .scope(world)
            .exec(lua)
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_yields_all_entities_with_the_queried_component() {
        let (runtime, mut world) = setup();
        world.spawn(TestHealth { value: 10 });
        world.spawn(TestHealth { value: 20 });
        world.spawn_empty();

        run(
            &runtime,
            &mut world,
            "
            local count = 0
            for id, health in world:query('TestHealth'):iter() do
                count = count + 1
            end
            assert(count == 2, 'expected 2, got ' .. count)
        ",
        );
    }

    #[test]
    fn iter_yields_component_values() {
        let (runtime, mut world) = setup();
        world.spawn(TestHealth { value: 77 });

        run(
            &runtime,
            &mut world,
            "
            for id, health in world:query('TestHealth'):iter() do
                assert(health.value == 77, 'expected 77, got ' .. tostring(health.value))
            end
        ",
        );
    }

    #[test]
    fn iter_is_empty_when_no_entity_has_the_component() {
        let (runtime, mut world) = setup();
        world.spawn_empty();

        run(
            &runtime,
            &mut world,
            "
            local count = 0
            for id, health in world:query('TestHealth'):iter() do
                count = count + 1
            end
            assert(count == 0)
        ",
        );
    }

    #[test]
    fn iter_requires_all_queried_components() {
        let (runtime, mut world) = setup();
        world.spawn(TestHealth { value: 1 });
        world.spawn(TestPosition { x: 0, y: 0 });
        world.spawn((TestHealth { value: 2 }, TestPosition { x: 1, y: 1 }));

        run(
            &runtime,
            &mut world,
            "
            local count = 0
            for id, health, pos in world:query('TestHealth', 'TestPosition'):iter() do
                count = count + 1
            end
            assert(count == 1, 'expected 1, got ' .. count)
        ",
        );
    }

    #[test]
    fn iter_yields_entity_id_as_first_value() {
        let (runtime, mut world) = setup();
        let entity = world.spawn(TestHealth { value: 0 }).id();

        run(
            &runtime,
            &mut world,
            &format!(
                "
            local expected_id = {expected}
            for id, health in world:query('TestHealth'):iter() do
                assert(id == expected_id, 'expected ' .. expected_id .. ', got ' .. id)
            end
        ",
                expected = entity.to_bits() as i64
            ),
        );
    }

    #[test]
    fn iter_is_empty_for_unknown_component_name() {
        let (runtime, mut world) = setup();
        world.spawn(TestHealth { value: 1 });

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
    fn iter_with_two_components_yields_both_values_per_row() {
        let (runtime, mut world) = setup();
        world.spawn((TestHealth { value: 7 }, TestPosition { x: 3, y: 4 }));

        run(
            &runtime,
            &mut world,
            "
            for id, health, pos in world:query('TestHealth', 'TestPosition'):iter() do
                assert(health.value == 7,  'health.value expected 7, got '  .. \
             tostring(health.value))
                assert(pos.x      == 3,   'pos.x expected 3, got '          .. tostring(pos.x))
                assert(pos.y      == 4,   'pos.y expected 4, got '          .. tostring(pos.y))
            end
        ",
        );
    }

    #[test]
    fn iter_reflects_values_after_lua_set() {
        let (runtime, mut world) = setup();
        world
            .resource_mut::<ScriptRegistry>()
            .components
            .get_mut("TestHealth")
            .expect("TestHealth accessor should be registered")
            .set = writable_health_accessor().set;

        world.spawn(TestHealth { value: 1 });

        run(
            &runtime,
            &mut world,
            "
            for id, health in world:query('TestHealth'):iter() do
                world:entity(id):set('TestHealth', { value = health.value + 10 })
            end
            for id, health in world:query('TestHealth'):iter() do
                assert(health.value == 11, 'expected 11, got ' .. tostring(health.value))
            end
        ",
        );
    }

    #[test]
    fn proxy_assignment_writes_component_back_to_ecs() {
        let (runtime, mut world) = setup();
        world
            .resource_mut::<ScriptRegistry>()
            .components
            .insert("TestHealth".to_string(), writable_health_accessor());

        let entity = world.spawn(TestHealth { value: 5 }).id();

        run(
            &runtime,
            &mut world,
            "
            for id, health in world:query('TestHealth'):iter() do
                health.value = health.value + 10
            end
        ",
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
    fn proxy_assignment_writes_multiple_fields_as_single_set() {
        use std::cell::Cell;

        thread_local! {
            static SET_COUNT: Cell<u32> = const { Cell::new(0) };
        }

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.resource_mut::<ScriptRegistry>().register_component(
            "TestPosition",
            ComponentAccessor {
                get: |entity, world| {
                    world
                        .get::<TestPosition>(entity)
                        .map(|p| serde_json::json!({ "x": p.x, "y": p.y }))
                },
                set: |entity, world, json| {
                    SET_COUNT.with(|c| c.set(c.get() + 1));
                    let x = json.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let y = json.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    world.entity_mut(entity).insert(TestPosition { x, y });
                },
                component_id: |world| world.register_component::<TestPosition>(),
            },
        );

        let entity = world.spawn(TestPosition { x: 1, y: 2 }).id();
        let runtime = LuaRuntime::new();

        run(
            &runtime,
            &mut world,
            "
            for id, pos in world:query('TestPosition'):iter() do
                pos.x = 10
                pos.y = 20
            end
        ",
        );

        let pos = world
            .get::<TestPosition>(entity)
            .expect("TestPosition should be present");
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 20);
        assert_eq!(
            SET_COUNT.with(|c| c.get()),
            1,
            "two field writes should produce exactly one component_set call"
        );
    }

    #[test]
    fn second_query_observes_proxy_writes_from_first_query() {
        let (runtime, mut world) = setup();
        world
            .resource_mut::<ScriptRegistry>()
            .components
            .insert("TestHealth".to_string(), writable_health_accessor());

        world.spawn(TestHealth { value: 1 });

        run(
            &runtime,
            &mut world,
            "
            for id, health in world:query('TestHealth'):iter() do
                health.value = health.value + 100
            end
            for id, health in world:query('TestHealth'):iter() do
                assert(health.value == 101, 'expected 101, got ' .. tostring(health.value))
            end
        ",
        );
    }

    #[test]
    fn proxy_dirty_flag_does_not_write_when_no_assignment_made() {
        use std::cell::Cell;

        thread_local! {
            static SET_COUNT: Cell<u32> = const { Cell::new(0) };
        }

        let (runtime, mut world) = setup();
        world
            .resource_mut::<ScriptRegistry>()
            .components
            .get_mut("TestHealth")
            .expect("TestHealth should be registered")
            .set = |_entity, _world, _json| {
            SET_COUNT.with(|c| c.set(c.get() + 1));
        };

        world.spawn(TestHealth { value: 42 });

        run(
            &runtime,
            &mut world,
            "
            for id, health in world:query('TestHealth'):iter() do
                local _ = health.value  -- read only, no assignment
            end
        ",
        );

        assert_eq!(
            SET_COUNT.with(|c| c.get()),
            0,
            "read-only iteration should not trigger any component_set calls"
        );
    }
}
