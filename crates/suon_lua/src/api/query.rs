//! [`QueryProxy`] — the Lua userdata returned by `Query(A, B, ...)`.
//!
//! Iterating via `:iter()` yields the entity id and one proxy table per component.
//! Writes to those proxy tables are batched and flushed to the ECS at the start of
//! the **next** iteration step rather than immediately (see the comment in `iter_fn`).

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
    api::{IntoJsonValueExt, IntoLuaValueExt},
    runtime::{QueryPlan, ScriptRegistry},
    world_cell,
};

/// `(component_id_fn, getter_fn, setter_fn)` — local alias used while building a `QueryPlan`.
type ComponentEntry = (
    fn(&mut World) -> ComponentId,
    fn(Entity, &mut World) -> Option<serde_json::Value>,
    fn(Entity, &mut World, serde_json::Value),
);

/// One component column in a query row: the snapshot JSON and its setter.
type RowComponent = (serde_json::Value, fn(Entity, &mut World, serde_json::Value));

/// Pending write held across iteration steps: data table, dirty flag, entity, setter.
type PendingWrite = (
    Rc<mlua::Table>,
    Rc<Cell<bool>>,
    Entity,
    fn(Entity, &mut World, serde_json::Value),
);

/// Extension for running dynamic ECS queries from the Lua side.
pub(crate) trait WorldLuaQueryExt {
    /// Runs the query described by `component_names` and materialises all matching rows.
    fn lua_query(&mut self, component_names: &[String]) -> Vec<(u64, Vec<RowComponent>)>;
}

impl WorldLuaQueryExt for World {
    fn lua_query(&mut self, component_names: &[String]) -> Vec<(u64, Vec<RowComponent>)> {
        let mut query_rows = Vec::new();

        self.resource_scope(|world, mut registry: Mut<ScriptRegistry>| {
            let cache_key = component_names.to_vec();
            let query_plan = match registry.query_cache.get(&cache_key) {
                Some(cached_plan) => cached_plan.clone(),
                None => {
                    let component_entries: Vec<ComponentEntry> = component_names
                        .iter()
                        .filter_map(|component_name| {
                            registry
                                .components
                                .get(component_name.as_str())
                                .map(|accessor| (accessor.component_id, accessor.get, accessor.set))
                        })
                        .collect();

                    let resolved_plan = if component_entries.len() != component_names.len() {
                        None
                    } else {
                        let component_ids: Vec<ComponentId> = component_entries
                            .iter()
                            .map(|(component_id_fn, _, _)| component_id_fn(world))
                            .collect();

                        if component_ids.is_empty() {
                            None
                        } else {
                            Some(QueryPlan {
                                component_ids,
                                get_fns: component_entries
                                    .iter()
                                    .map(|(_, getter, _)| *getter)
                                    .collect(),
                                set_fns: component_entries
                                    .iter()
                                    .map(|(_, _, setter)| *setter)
                                    .collect(),
                            })
                        }
                    };

                    registry
                        .query_cache
                        .insert(cache_key, resolved_plan.clone());
                    resolved_plan
                }
            };

            let Some(query_plan) = query_plan else { return };

            let mut query_builder = QueryBuilder::<Entity>::new(world);
            for &component_id in &query_plan.component_ids {
                query_builder.with_id(component_id);
            }

            let mut query_state = query_builder.build();
            let matching_entities: Vec<Entity> = query_state.iter(world).collect();

            for entity in matching_entities {
                let row_components: Vec<RowComponent> = query_plan
                    .get_fns
                    .iter()
                    .zip(query_plan.set_fns.iter())
                    .filter_map(|(getter, setter)| {
                        getter(entity, world).map(|component_json| (component_json, *setter))
                    })
                    .collect();
                query_rows.push((entity.to_bits(), row_components));
            }
        });

        query_rows
    }
}

/// Lua UserData returned by `Query(A, B, ...)`.
///
/// ```lua
/// for id, hp in Query(Health):iter() do
///     hp.value = hp.value - 1   -- batched: written to ECS once per entity at the next step
/// end
///
/// for id, hp, pos in Query(Health, Position):iter() do
///     hp.value  = 0
///     pos.x     = 0
/// end
/// ```
pub struct QueryProxy {
    pub(crate) components: Vec<String>,
}

impl UserData for QueryProxy {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("iter", |lua, this, ()| {
            let component_names = this.components.clone();
            let query_rows = world_cell::with(|world| world.lua_query(&component_names));

            let query_rows = Rc::new(query_rows);
            let row_cursor = Rc::new(Cell::new(0usize));
            let pending_writes: Rc<RefCell<Vec<PendingWrite>>> = Rc::new(RefCell::new(Vec::new()));

            let iterator_function = lua.create_function(move |lua, _: mlua::MultiValue| {
                // Flush writes from the previous step before advancing the cursor.
                // We flush here (start of step N+1) rather than at the end of step N
                // because Lua reads the __newindex result immediately after assignment —
                // deferring until the next call lets the current step finish without
                // a re-entrant world borrow.
                for (component_data, is_dirty, entity, setter) in pending_writes.borrow().iter() {
                    if is_dirty.get() {
                        is_dirty.set(false);
                        let updated_json =
                            mlua::Value::Table((**component_data).clone()).into_json_value()?;
                        world_cell::with(|world| setter(*entity, world, updated_json));
                    }
                }
                pending_writes.borrow_mut().clear();

                let row_index = row_cursor.get();
                if row_index >= query_rows.len() {
                    return Ok(mlua::MultiValue::new());
                }
                row_cursor.set(row_index + 1);

                let (entity_bits, ref row_components) = query_rows[row_index];
                let entity = Entity::from_bits(entity_bits);

                let mut iterator_values = mlua::MultiValue::new();
                iterator_values.push_back(mlua::Value::Integer(entity_bits as i64));

                for (component_json, setter) in row_components.iter() {
                    let component_data = Rc::new(component_json.clone().into_lua_table(lua)?);
                    let is_dirty = Rc::new(Cell::new(false));
                    let setter = *setter;

                    pending_writes.borrow_mut().push((
                        component_data.clone(),
                        is_dirty.clone(),
                        entity,
                        setter,
                    ));

                    let proxy_table = lua.create_table()?;
                    let metatable = lua.create_table()?;

                    let index_component_data = component_data.clone();
                    metatable.set(
                        "__index",
                        lua.create_function(
                            move |_lua, (_proxy_table, key): (mlua::Table, String)| {
                                index_component_data.raw_get::<mlua::Value>(key)
                            },
                        )?,
                    )?;

                    let newindex_component_data = component_data.clone();
                    let newindex_is_dirty = is_dirty.clone();
                    metatable.set(
                        "__newindex",
                        lua.create_function(
                            move |_lua,
                                  (_proxy_table, key, lua_value): (
                                mlua::Table,
                                String,
                                mlua::Value,
                            )| {
                                newindex_component_data.raw_set(key, lua_value)?;
                                newindex_is_dirty.set(true);
                                Ok(())
                            },
                        )?,
                    )?;

                    proxy_table.set_metatable(Some(metatable))?;
                    iterator_values.push_back(mlua::Value::Table(proxy_table));
                }

                Ok(iterator_values)
            })?;

            Ok(iterator_function)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lua_component::LuaComponent,
        runtime::{ComponentAccessor, LuaRuntime, ScriptRegistry},
    };
    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};
    use suon_macros::LuaComponent;

    #[derive(LuaComponent, Serialize, Deserialize)]
    struct Health {
        value: i32,
    }

    #[derive(LuaComponent, Serialize, Deserialize)]
    struct Position {
        x: i32,
        y: i32,
    }

    #[test]
    fn iter_yields_all_entities_with_the_queried_component() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 10 });
        world.spawn(Health { value: 20 });
        world.spawn_empty();

        runtime
            .scope(&mut world)
            .execute(
                "
            local count = 0
            for id, health in Query(Health):iter() do
                count = count + 1
            end
            assert(count == 2, 'expected 2, got ' .. count)
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_yields_component_values() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 77 });

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health in Query(Health):iter() do
                assert(health.value == 77, 'expected 77, got ' .. tostring(health.value))
            end
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_is_empty_when_no_entity_has_the_component() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        // Spawn then despawn to register the Health global without leaving any entity.
        let entity = world.spawn(Health { value: 0 }).id();
        world.despawn(entity);

        runtime
            .scope(&mut world)
            .execute(
                "
            local count = 0
            for id, health in Query(Health):iter() do
                count = count + 1
            end
            assert(count == 0)
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_requires_all_queried_components() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 1 });
        world.spawn(Position { x: 0, y: 0 });
        world.spawn((Health { value: 2 }, Position { x: 1, y: 1 }));

        runtime
            .scope(&mut world)
            .execute(
                "
            local count = 0
            for id, health, pos in Query(Health, Position):iter() do
                count = count + 1
            end
            assert(count == 1, 'expected 1, got ' .. count)
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_yields_entity_id_as_first_value() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 0 }).id();

        runtime
            .scope(&mut world)
            .execute(&format!(
                "
            local expected_id = {expected}
            for id, health in Query(Health):iter() do
                assert(id == expected_id, 'expected ' .. expected_id .. ', got ' .. id)
            end
        ",
                expected = entity.to_bits() as i64
            ))
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_with_duplicate_component_names_returns_both_columns() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 33 });

        runtime
            .scope(&mut world)
            .execute(
                "
            local count = 0
            for id, health_a, health_b in Query(Health, Health):iter() do
                count = count + 1
                assert(health_a.value == 33)
                assert(health_b.value == 33)
            end
            assert(count == 1, 'expected 1, got ' .. count)
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_with_two_components_yields_both_values_per_row() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn((Health { value: 7 }, Position { x: 3, y: 4 }));

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health, pos in Query(Health, Position):iter() do
                assert(health.value == 7,  'health.value expected 7, got '  .. \
                 tostring(health.value))
                assert(pos.x      == 3,   'pos.x expected 3, got '          .. tostring(pos.x))
                assert(pos.y      == 4,   'pos.y expected 4, got '          .. tostring(pos.y))
            end
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn iter_reflects_values_after_proxy_write() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 1 });

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health in Query(Health):iter() do
                health.value = health.value + 10
            end
            for id, health in Query(Health):iter() do
                assert(health.value == 11, 'expected 11, got ' .. tostring(health.value))
            end
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn proxy_assignment_writes_component_back_to_ecs() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Health { value: 5 }).id();

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health in Query(Health):iter() do
                health.value = health.value + 10
            end
        ",
            )
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
    fn proxy_assignment_writes_multiple_fields_as_single_set() {
        use std::cell::Cell;

        thread_local! {
            static SET_COUNT: Cell<u32> = const { Cell::new(0) };
        }

        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(Position { x: 1, y: 2 }).id();

        world.resource_mut::<ScriptRegistry>().components.insert(
            Position::lua_name().to_string(),
            ComponentAccessor {
                set: |entity, world, json| {
                    SET_COUNT.with(|c| c.set(c.get() + 1));
                    let x = json.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    let y = json.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                    world.entity_mut(entity).insert(Position { x, y });
                },
                ..Position::make_accessor()
            },
        );

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, pos in Query(Position):iter() do
                pos.x = 10
                pos.y = 20
            end
        ",
            )
            .expect("lua exec should succeed");

        let position = world
            .get::<Position>(entity)
            .expect("Position should be present");

        assert_eq!(position.x, 10);

        assert_eq!(position.y, 20);

        assert_eq!(
            SET_COUNT.with(|c| c.get()),
            1,
            "two field writes should produce exactly one deserialize_component call"
        );
    }

    #[test]
    fn second_query_observes_proxy_writes_from_first_query() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 1 });

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health in Query(Health):iter() do
                health.value = health.value + 100
            end
            for id, health in Query(Health):iter() do
                assert(health.value == 101, 'expected 101, got ' .. tostring(health.value))
            end
        ",
            )
            .expect("lua exec should succeed");
    }

    #[test]
    fn proxy_dirty_flag_does_not_write_when_no_assignment_made() {
        use std::cell::Cell;

        thread_local! {
            static SET_COUNT: Cell<u32> = const { Cell::new(0) };
        }

        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        world.spawn(Health { value: 42 });

        world
            .resource_mut::<ScriptRegistry>()
            .components
            .get_mut("Health")
            .expect("Health should be registered")
            .set = |_entity, _world, _json| {
            SET_COUNT.with(|c| c.set(c.get() + 1));
        };

        runtime
            .scope(&mut world)
            .execute(
                "
            for id, health in Query(Health):iter() do
                local _ = health.value  -- read only, no assignment
            end
        ",
            )
            .expect("lua exec should succeed");

        assert_eq!(
            SET_COUNT.with(|c| c.get()),
            0,
            "read-only iteration should not trigger any deserialize_component calls"
        );
    }
}
