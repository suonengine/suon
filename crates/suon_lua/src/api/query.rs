use bevy::{
    ecs::{component::ComponentId, query::QueryBuilder},
    prelude::*,
};
use mlua::{UserData, UserDataMethods};
use std::{cell::Cell, rc::Rc};

use crate::{api::json_to_lua, runtime::ScriptRegistry, world_cell};

type ComponentEntry = (
    fn(&mut World) -> ComponentId,
    fn(Entity, &mut World) -> Option<serde_json::Value>,
);

/// Lua UserData returned by `world:query(...)`.
///
/// ```lua
/// for id, hp, pos in world:query("Health", "Position"):iter() do
///     if hp.value < 10 then
///         world:entity(id):trigger("TeleportIntent", { to = { x=0, y=0 } })
///     end
/// end
/// ```
pub struct QueryProxy {
    pub(crate) components: Vec<String>,
}

impl UserData for QueryProxy {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("iter", |lua, this, ()| {
            let component_names = this.components.clone();

            // Collect all (entity_bits, [comp_json, ...]) rows from the world.
            let rows = world_cell::with(|world| collect_query(world, &component_names));

            // Wrap in Rc so the stateful closure can own it without requiring Send.
            let rows = Rc::new(rows);
            let cursor = Rc::new(Cell::new(0usize));

            // Return a stateful Lua function usable in `for ... in` loops.
            let iter_fn = lua.create_function(move |lua, _: mlua::MultiValue| {
                let index = cursor.get();
                if index >= rows.len() {
                    return Ok(mlua::MultiValue::new());
                }
                cursor.set(index + 1);

                let (entity_bits, ref component_values) = rows[index];
                let mut multi_value = mlua::MultiValue::new();
                multi_value.push_back(mlua::Value::Integer(entity_bits as i64));
                for component_json in component_values {
                    multi_value.push_back(json_to_lua(lua, component_json.clone())?);
                }
                Ok(multi_value)
            })?;

            Ok(iter_fn)
        });
    }
}

/// Runs the query against the world and collects results.
///
/// Uses [`resource_scope`] so the registry and world can be borrowed independently,
/// then builds a dynamic [`QueryState`] via [`QueryBuilder`].
fn collect_query(
    world: &mut World,
    component_names: &[String],
) -> Vec<(u64, Vec<serde_json::Value>)> {
    let mut results = Vec::new();

    world.resource_scope(|world, registry: Mut<ScriptRegistry>| {
        // Collect fn ptrs from registry — borrow ends after collect().
        let entries: Vec<ComponentEntry> = component_names
            .iter()
            .filter_map(|name| {
                registry
                    .components
                    .get(name.as_str())
                    .map(|accessor| (accessor.component_id, accessor.get))
            })
            .collect();

        // Resolve ComponentIds using world (registry no longer borrowed).
        let component_ids: Vec<ComponentId> =
            entries.iter().map(|(init_id, _)| init_id(world)).collect();

        let get_fns: Vec<fn(Entity, &mut World) -> Option<serde_json::Value>> =
            entries.iter().map(|(_, get)| *get).collect();

        if component_ids.is_empty() {
            return;
        }

        // Dynamic query — entities that have all requested components.
        let mut builder = QueryBuilder::<Entity>::new(world);
        for &component_id in &component_ids {
            builder.with_id(component_id);
        }

        let mut query_state = builder.build();
        let entities: Vec<Entity> = query_state.iter(world).collect();

        // Fetch component values per entity.
        for entity in entities {
            let component_values: Vec<serde_json::Value> = get_fns
                .iter()
                .filter_map(|get| get(entity, world))
                .collect();
            results.push((entity.to_bits(), component_values));
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
        world.spawn(TestHealth { value: 1 }); // health only
        world.spawn(TestPosition { x: 0, y: 0 }); // position only
        world.spawn((TestHealth { value: 2 }, TestPosition { x: 1, y: 1 })); // both

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

        // Register a writable accessor for TestHealth
        world
            .resource_mut::<ScriptRegistry>()
            .components
            .get_mut("TestHealth")
            .expect("TestHealth accessor should be registered")
            .set = |entity, world, json| {
            if let Some(value) = json.get("value").and_then(|v| v.as_i64()) {
                world.entity_mut(entity).insert(TestHealth {
                    value: value as i32,
                });
            }
        };

        world.spawn(TestHealth { value: 1 });

        run(
            &runtime,
            &mut world,
            "
            -- Bump every entity's health by 10
            for id, health in world:query('TestHealth'):iter() do
                world:entity(id):set('TestHealth', { value = health.value + 10 })
            end
            -- Re-query and verify the new value
            for id, health in world:query('TestHealth'):iter() do
                assert(health.value == 11, 'expected 11, got ' .. tostring(health.value))
            end
        ",
        );
    }
}
