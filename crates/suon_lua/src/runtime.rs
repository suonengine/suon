//! Core Lua runtime: [`LuaRuntime`], [`LuaScope`], [`ScriptRegistry`], and accessors.
//!
//! [`LuaRuntime`] owns the VM; [`LuaScope`] combines it with exclusive world access
//! so Lua callbacks can reach Bevy components safely. [`ScriptRegistry`] maps string
//! names to type-erased get/set/trigger function pointers.

use bevy::{ecs::component::ComponentId, prelude::*};
use mlua::Function;
use serde_json::Value as Json;
use std::{cell::RefCell, collections::HashMap};

use crate::{
    api::{IntoLuaValueExt, entity::EntityProxy, world::LuaWorldApiExt},
    world_cell::{self, WorldContext},
};

/// Non-send Bevy resource that owns the mlua Lua VM.
///
/// There is exactly one `LuaRuntime` per Bevy world, inserted by [`crate::LuaPlugin`].
/// Because mlua's `Lua` is `!Send`, the runtime is stored as a non-send resource
/// and must be accessed from the main thread only.
///
/// Use [`LuaRuntime::scope`] to execute Lua code, or [`WorldLuaRuntimeExt::lua_runtime`]
/// when you need to hold `&mut World` alongside the runtime.
///
/// # Examples
///
/// ```rust,ignore
/// # use bevy::prelude::*;
/// # use suon_lua::LuaRuntime;
/// let mut world = World::new();
/// world.insert_non_send_resource(LuaRuntime::new());
/// ```
pub struct LuaRuntime {
    /// Shared mlua state used by every script execution in the world.
    lua: mlua::Lua,
    /// Cache of compiled chunks keyed by their exact source text.
    script_cache: RefCell<HashMap<Box<str>, mlua::RegistryKey>>,
}

impl LuaRuntime {
    /// Creates a new runtime and registers the global Lua API surface.
    pub(crate) fn new() -> Self {
        let lua = mlua::Lua::new();
        if let Err(error) = lua.register_world_api() {
            panic!("failed to register Lua globals: {error}");
        }
        Self {
            lua,
            script_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Enters a [`LuaScope`] that gives Lua callbacks access to `world` for its lifetime.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaRuntime;
    /// let runtime = LuaRuntime::new();
    /// let mut world = World::new();
    /// runtime.scope(&mut world).execute("x = 1")?;
    /// # Ok::<(), mlua::Error>(())
    /// ```
    pub fn scope<'runtime, 'world>(
        &'runtime self,
        world: &'world mut World,
    ) -> LuaScope<'runtime, 'world> {
        LuaScope {
            lua: &self.lua,
            script_cache: &self.script_cache,
            _context: WorldContext::enter(world),
        }
    }
}

/// Extension methods for accessing the non-send [`LuaRuntime`] straight from [`World`].
pub trait WorldLuaRuntimeExt {
    /// Temporarily removes the [`LuaRuntime`] resource, runs `callback`, then re-inserts it.
    ///
    /// Returns `None` when the world does not contain a runtime.
    fn lua_runtime<R>(&mut self, callback: impl FnOnce(&LuaRuntime, &mut World) -> R) -> Option<R>;
}

impl WorldLuaRuntimeExt for World {
    fn lua_runtime<R>(&mut self, callback: impl FnOnce(&LuaRuntime, &mut World) -> R) -> Option<R> {
        let runtime = self.remove_non_send_resource::<LuaRuntime>()?;
        let result = callback(&runtime, self);
        self.insert_non_send_resource(runtime);
        Some(result)
    }
}

/// Short-lived execution context that pairs the Lua VM with exclusive ECS access.
///
/// Created by [`LuaRuntime::scope`]. For its lifetime the raw world pointer in
/// `world_cell` is valid, so Lua callbacks triggered by [`execute`] or
/// [`call_hook`] can safely call `world_cell::with`.
///
/// Dropping `LuaScope` clears the world pointer — any Lua callback that outlives
/// this scope will panic if it tries to access the world.
///
/// [`execute`]: LuaScope::execute
/// [`call_hook`]: LuaScope::call_hook
///
/// # Examples
///
/// ```rust,ignore
/// # use bevy::prelude::*;
/// # use suon_lua::LuaRuntime;
/// let runtime = LuaRuntime::new();
/// let mut world = World::new();
/// let scope = runtime.scope(&mut world);
/// scope.execute("flag = true")?;
/// # Ok::<(), mlua::Error>(())
/// ```
pub struct LuaScope<'runtime, 'world> {
    /// Borrow of the shared Lua VM owned by [`LuaRuntime`].
    lua: &'runtime mlua::Lua,
    /// Shared cache of compiled Lua chunks for this runtime.
    script_cache: &'runtime RefCell<HashMap<Box<str>, mlua::RegistryKey>>,
    /// Guard that keeps the current Bevy world reachable from Lua callbacks.
    _context: WorldContext<'world>,
}

impl LuaScope<'_, '_> {
    /// Drains `ScriptRegistry::pending_globals` and creates a Lua global table for each name.
    ///
    /// Each global looks like `Health = { __component = "Health" }`, which lets scripts
    /// pass the bare identifier to `Query`: `Query(Health, Position)`.
    fn sync_pending_globals(&self) -> mlua::Result<()> {
        let pending_component_names = world_cell::with(|world| {
            std::mem::take(&mut world.resource_mut::<ScriptRegistry>().pending_globals)
        });

        if pending_component_names.is_empty() {
            return Ok(());
        }

        let globals = self.lua.globals();
        for component_name in pending_component_names {
            let component_global_table = self.lua.create_table()?;
            component_global_table.set("__component", component_name.clone())?;
            globals.set(component_name, component_global_table)?;
        }
        Ok(())
    }

    /// Returns a compiled function for `source`, using the cache when possible.
    fn compiled_chunk(&self, source: &str) -> mlua::Result<Function> {
        if let Some(cache_key) = self.script_cache.borrow().get(source) {
            return self.lua.registry_value(cache_key);
        }

        let function = self.lua.load(source).into_function()?;
        let registry_key = self.lua.create_registry_value(function.clone())?;

        self.script_cache
            .borrow_mut()
            .insert(source.into(), registry_key);

        Ok(function)
    }

    /// Compiles and executes `source` as a Lua chunk in the shared VM state.
    ///
    /// # Errors
    ///
    /// Returns `mlua::Error::SyntaxError` for invalid Lua, or
    /// `mlua::Error::RuntimeError` if the chunk calls `error(...)`.
    /// The VM state is **not** rolled back on error — globals set before the
    /// error remain visible in subsequent calls.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaRuntime;
    /// let runtime = LuaRuntime::new();
    /// let mut world = World::new();
    /// runtime.scope(&mut world).execute("counter = 1")?;
    /// # Ok::<(), mlua::Error>(())
    /// ```
    pub fn execute(&self, source: &str) -> mlua::Result<()> {
        self.sync_pending_globals()?;
        self.compiled_chunk(source)?.call::<()>(())
    }

    /// Converts serialized Rust hook arguments into the Lua argument list.
    fn hook_arguments(
        &self,
        entity_proxy: mlua::AnyUserData,
        args: Json,
    ) -> mlua::Result<mlua::MultiValue> {
        let mut lua_arguments = mlua::MultiValue::new();
        lua_arguments.push_back(mlua::Value::UserData(entity_proxy));

        match args {
            Json::Null => {}
            Json::Array(array_items) => {
                for array_item in array_items {
                    lua_arguments.push_back(array_item.into_lua_value(self.lua)?);
                }
            }
            scalar_value => lua_arguments.push_back(scalar_value.into_lua_value(self.lua)?),
        }

        Ok(lua_arguments)
    }

    /// Loads `source`, then calls `hook` as a method on the global `Entity` table.
    ///
    /// Scripts are expected to define hooks in method form, for example
    /// 1. `Entity.<hook>` — called as a method: `Entity:onTick(proxy)`
    /// 2. global `<hook>` — called as a plain function: `onTick(proxy)`
    ///
    /// If neither form exists the call is a silent no-op (not an error).
    ///
    /// # Errors
    ///
    /// Returns an error if `source` has a syntax error or if the hook function
    /// itself calls `error(...)`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaRuntime;
    /// let runtime = LuaRuntime::new();
    /// let mut world = World::new();
    /// let entity = world.spawn_empty().id();
    ///
    /// runtime.scope(&mut world).call_hook(
    ///     entity,
    ///     "function Entity:onTick() touched = true end",
    ///     "onTick",
    ///     serde_json::Value::Null,
    /// )?;
    /// # Ok::<(), mlua::Error>(())
    /// ```
    pub fn call_hook(
        &self,
        entity: Entity,
        source: &str,
        hook: &str,
        args: Json,
    ) -> mlua::Result<()> {
        self.sync_pending_globals()?;
        self.compiled_chunk(source)?.call::<()>(())?;

        let globals = self.lua.globals();
        let entity_proxy = self.lua.create_userdata(EntityProxy { id: entity })?;
        let lua_arguments = self.hook_arguments(entity_proxy, args)?;

        // Hooks are method-only so scripts have a single, predictable convention:
        // `function Entity:onTick() ... end`.
        if let Ok(entity_class) = globals.get::<mlua::Table>("Entity")
            && let Ok(hook_function) = entity_class.get::<Function>(hook)
        {
            hook_function.call::<()>(lua_arguments)?;
        }

        Ok(())
    }
}

/// Type-erased vtable for a single component type registered in [`ScriptRegistry`].
///
/// All three fields must be consistent — they should all refer to the same `T`:
/// - `get` serialises `T` to JSON (returns `None` when the entity lacks the component)
/// - `set` deserialises JSON and inserts/replaces `T` on the entity
/// - `component_id` registers `T` with the world and returns its stable [`ComponentId`]
///
/// Construct via [`crate::prelude::LuaComponent::make_accessor`] or closures that call
/// [`crate::prelude::WorldLuaComponentExt::serialize_lua_component`],
/// [`crate::prelude::WorldLuaComponentExt::deserialize_lua_component`], and
/// `World::register_component::<T>()`.
///
/// # Examples
///
/// ```rust,ignore
/// # use bevy::prelude::*;
/// # use serde::{Deserialize, Serialize};
/// # use suon_lua::{ComponentAccessor, WorldLuaComponentExt};
/// #[derive(Component, Serialize, Deserialize)]
/// struct Health {
///     value: i32,
/// }
///
/// let accessor = ComponentAccessor {
///     get: |entity, world| world.serialize_lua_component::<Health>(entity),
///     set: |entity, world, json| world.deserialize_lua_component::<Health>(entity, json),
///     component_id: |world| world.register_component::<Health>(),
/// };
/// # let _ = accessor;
/// ```
pub struct ComponentAccessor {
    /// Serializes the component to JSON. Returns `None` if the entity lacks the component.
    pub get: fn(Entity, &mut World) -> Option<Json>,

    /// Deserializes JSON and inserts/updates the component on the entity.
    pub set: fn(Entity, &mut World, Json),

    /// Returns the [`ComponentId`] for this component type, registering it if needed.
    pub component_id: fn(&mut World) -> ComponentId,
}

/// Type-erased vtable for a trigger registered in [`ScriptRegistry`].
///
/// `fire` receives the entity, the world, and the args table serialised to JSON.
///
/// # Examples
///
/// ```rust,ignore
/// # use bevy::prelude::*;
/// # use suon_lua::TriggerAccessor;
/// let trigger = TriggerAccessor {
///     fire: |_entity, _world, _args| {},
/// };
/// # let _ = trigger;
/// ```
pub struct TriggerAccessor {
    /// Deserializes the args table and fires the trigger on the entity.
    pub fire: fn(Entity, &mut World, Json),
}

/// Bevy resource that maps Lua-visible names to component and trigger accessors.
///
/// Components are registered automatically the first time a
/// [`crate::prelude::LuaComponent`] is inserted into any entity via the `on_add`
/// hook generated by `#[derive(LuaComponent)]`. Manual registration via
/// [`register_component`] or [`crate::prelude::AppLuaExt::register_lua_component`]
/// is needed when you want the component available before the first insert.
///
/// [`register_component`]: ScriptRegistry::register_component
///
/// # Examples
///
/// ```rust,ignore
/// # use suon_lua::ScriptRegistry;
/// let registry = ScriptRegistry::default();
/// # let _ = registry;
/// ```
#[derive(Resource, Default)]
pub struct ScriptRegistry {
    /// Component accessors keyed by their Lua-visible names.
    pub(crate) components: HashMap<String, ComponentAccessor>,

    /// Trigger accessors keyed by their Lua-visible names.
    pub(crate) triggers: HashMap<String, TriggerAccessor>,

    /// Cached query plans keyed by the requested component name list.
    pub(crate) query_cache: HashMap<Vec<String>, Option<QueryPlan>>,

    /// Names of components registered since the last Lua execution, waiting to be
    /// promoted to Lua globals so scripts can write `Query(Health, Position)`.
    pub(crate) pending_globals: Vec<String>,
}

impl ScriptRegistry {
    /// Returns `true` if a component accessor is registered under `name`.
    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// Registers a Lua-visible component accessor under `name`.
    pub fn register_component(&mut self, name: impl Into<String>, accessor: ComponentAccessor) {
        let name = name.into();
        self.pending_globals.push(name.clone());
        self.components.insert(name, accessor);
        self.query_cache.clear();
    }

    /// Registers a Lua-visible trigger accessor under `name`.
    pub fn register_trigger(&mut self, name: impl Into<String>, accessor: TriggerAccessor) {
        self.triggers.insert(name.into(), accessor);
    }
}

#[derive(Clone)]
pub(crate) struct QueryPlan {
    /// Registered Bevy component ids used to build the dynamic query.
    pub(crate) component_ids: Vec<ComponentId>,

    /// Component serializers used to materialize Lua-visible row values.
    pub(crate) get_fns: Vec<fn(Entity, &mut World) -> Option<Json>>,

    /// Component deserializers used to flush proxy writes back into the ECS.
    pub(crate) set_fns: Vec<fn(Entity, &mut World, Json)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registers_entity_global_in_lua() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        runtime
            .scope(&mut world)
            .execute("assert(Entity ~= nil)")
            .expect("lua exec should succeed");
    }

    #[test]
    fn exec_runs_lua_code() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let scope = runtime.scope(&mut world);

        scope
            .execute("result = 1 + 2")
            .expect("lua exec should succeed");

        scope
            .execute("assert(result == 3)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn exec_returns_error_on_syntax_error() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let error = runtime
            .scope(&mut world)
            .execute("this is !! not lua")
            .unwrap_err();

        assert!(
            error.to_string().to_lowercase().contains("syntax")
                || matches!(error, mlua::Error::SyntaxError { .. }),
            "unexpected error kind: {error}"
        );
    }

    #[test]
    fn exec_returns_error_on_runtime_error() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let error = runtime
            .scope(&mut world)
            .execute("error('intentional')")
            .unwrap_err();

        assert!(error.to_string().contains("intentional"));
    }

    #[test]
    fn call_hook_entity_method_style() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function Entity:onTick() ran = true end",
                "onTick",
                Json::Null,
            )
            .expect("hook should execute without error");

        scope
            .execute("assert(ran == true)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_ignores_plain_global_function() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function onTick(entity) ran = true end",
                "onTick",
                Json::Null,
            )
            .expect("hook should execute without error");
        scope
            .execute("assert(ran == nil)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_missing_hook_is_noop() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .call_hook(entity, "", "nonexistent", Json::Null)
            .expect("missing hook should be a noop, not an error");
    }

    #[test]
    fn call_hook_uses_only_entity_method_style() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "
            function Entity:onTick() style = 'method' end
            function onTick(entity)  style = 'plain'  end
        ",
                "onTick",
                Json::Null,
            )
            .expect("hook should execute without error");

        scope
            .execute("assert(style == 'method')")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_passes_entity_proxy_as_self() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function Entity:onTick() received_id = self:id() end",
                "onTick",
                Json::Null,
            )
            .expect("hook should execute without error");

        scope
            .execute(&format!("assert(received_id == {})", entity.to_bits()))
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_passes_single_table_argument() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function Entity:onMove(position) moved_x = position.x; moved_y = position.y end",
                "onMove",
                serde_json::json!({ "x": 5, "y": 9 }),
            )
            .expect("hook should execute without error");

        scope
            .execute("assert(moved_x == 5 and moved_y == 9)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_expands_array_into_multiple_arguments() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function Entity:onDamage(amount, source) damage = amount; dealer = source end",
                "onDamage",
                serde_json::json!([12, "lava"]),
            )
            .expect("hook should execute without error");

        scope
            .execute("assert(damage == 12 and dealer == 'lava')")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn exec_state_persists_across_multiple_calls_on_same_scope() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let scope = runtime.scope(&mut world);

        scope
            .execute("counter = 0")
            .expect("lua exec should succeed");

        scope
            .execute("counter = counter + 1")
            .expect("lua exec should succeed");

        scope
            .execute("counter = counter + 1")
            .expect("lua exec should succeed");

        scope
            .execute("assert(counter == 2)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_returns_error_when_hook_itself_errors() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let scope = runtime.scope(&mut world);

        let error = scope
            .call_hook(
                entity,
                "function Entity:onTick() error('hook failed') end",
                "onTick",
                Json::Null,
            )
            .unwrap_err();

        assert!(error.to_string().contains("hook failed"));
    }

    #[test]
    fn exec_empty_string_succeeds() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        runtime
            .scope(&mut world)
            .execute("")
            .expect("empty source should succeed");
    }

    #[test]
    fn globals_persist_across_separate_scope_calls_on_same_runtime() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        runtime
            .scope(&mut world)
            .execute("x = 42")
            .expect("lua execution should succeed");

        runtime
            .scope(&mut world)
            .execute("assert(x == 42)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn execute_caches_compiled_chunks_by_source() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        runtime
            .scope(&mut world)
            .execute("counter = 1")
            .expect("first lua execution should succeed");

        runtime
            .scope(&mut world)
            .execute("counter = 1")
            .expect("second lua execution should succeed");

        assert_eq!(runtime.script_cache.borrow().len(), 1);
    }

    #[test]
    fn call_hook_reuses_cached_script_chunk() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let source = "function Entity:onTick() touched = true end";

        runtime
            .scope(&mut world)
            .call_hook(entity, source, "onTick", Json::Null)
            .expect("cached chunk should execute successfully");

        runtime
            .scope(&mut world)
            .call_hook(entity, source, "onTick", Json::Null)
            .expect("cached chunk should execute successfully");

        assert_eq!(runtime.script_cache.borrow().len(), 1);
    }

    #[test]
    fn call_hook_with_empty_source_and_no_hook_is_noop() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .call_hook(entity, "", "onTick", Json::Null)
            .expect("empty source with no hook should be a noop");
    }

    #[test]
    fn call_hook_returns_error_when_source_has_syntax_error() {
        let runtime = LuaRuntime::new();

        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        let error = runtime
            .scope(&mut world)
            .call_hook(entity, "!! invalid !!", "onTick", Json::Null)
            .unwrap_err();

        assert!(matches!(error, mlua::Error::SyntaxError { .. }));
    }

    #[test]
    fn script_registry_is_empty_by_default() {
        let registry = ScriptRegistry::default();
        assert!(registry.components.is_empty());
        assert!(registry.triggers.is_empty());
    }

    #[test]
    fn register_component_inserts_into_map() {
        let mut registry = ScriptRegistry::default();

        registry.register_component(
            "Health",
            ComponentAccessor {
                get: |_, _| None,
                set: |_, _, _| {},
                component_id: |_| unreachable!(),
            },
        );

        assert!(registry.components.contains_key("Health"));
    }

    #[test]
    fn register_component_overwrites_existing_entry() {
        let mut registry = ScriptRegistry::default();

        for _ in 0..2 {
            registry.register_component(
                "Health",
                ComponentAccessor {
                    get: |_, _| None,
                    set: |_, _, _| {},
                    component_id: |_| unreachable!(),
                },
            );
        }

        assert_eq!(registry.components.len(), 1);
    }

    #[test]
    fn register_component_clears_query_cache() {
        let mut registry = ScriptRegistry::default();

        registry
            .query_cache
            .insert(vec!["Health".to_string()], None);

        registry.register_component(
            "Health",
            ComponentAccessor {
                get: |_, _| None,
                set: |_, _, _| {},
                component_id: |_| unreachable!(),
            },
        );

        assert!(registry.query_cache.is_empty());
    }

    #[test]
    fn register_trigger_inserts_into_map() {
        let mut registry = ScriptRegistry::default();

        registry.register_trigger("Heal", TriggerAccessor { fire: |_, _, _| {} });

        assert!(registry.triggers.contains_key("Heal"));
    }

    #[test]
    fn register_trigger_overwrites_existing_entry() {
        let mut registry = ScriptRegistry::default();

        for _ in 0..2 {
            registry.register_trigger("Heal", TriggerAccessor { fire: |_, _, _| {} });
        }

        assert_eq!(registry.triggers.len(), 1);
    }

    #[test]
    fn take_scope_returns_none_when_runtime_is_missing() {
        let mut world = World::new();
        let result = world.lua_runtime(|_, _| ());
        assert!(result.is_none());
    }

    #[test]
    fn take_scope_restores_runtime_after_call() {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.insert_non_send_resource(LuaRuntime::new());

        world.lua_runtime(|_, _| ());

        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn take_scope_restores_runtime_when_f_returns_err() {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.insert_non_send_resource(LuaRuntime::new());

        world.lua_runtime(|_, _| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError("test".into()))
        });

        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }
}
