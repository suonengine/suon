//! Core Lua runtime: [`LuaRuntime`], [`LuaScope`], [`ScriptRegistry`], and accessors.
//!
//! [`LuaRuntime`] owns the VM; [`LuaScope`] combines it with exclusive world access
//! so Lua callbacks can reach Bevy components safely. [`ScriptRegistry`] maps string
//! names to type-erased get/set/trigger function pointers.

use bevy::{ecs::component::ComponentId, prelude::*};
use mlua::Function;
use serde_json::Value as Json;
use std::collections::HashMap;

use crate::{api::entity::EntityProxy, world_cell::WorldContext};

/// Non-send Bevy resource that owns the mlua Lua VM.
///
/// There is exactly one `LuaRuntime` per Bevy world, inserted by [`crate::LuaPlugin`].
/// Because mlua's `Lua` is `!Send`, the runtime is stored as a non-send resource
/// and must be accessed from the main thread only.
///
/// Use [`LuaRuntime::scope`] to execute Lua code, or [`LuaRuntime::take_scope`]
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
    lua: mlua::Lua,
}

impl LuaRuntime {
    pub(crate) fn new() -> Self {
        let lua = mlua::Lua::new();
        crate::api::world::register_world_api(&lua).expect("failed to register Lua world API");
        Self { lua }
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
            _context: WorldContext::enter(world),
        }
    }

    /// Removes [`LuaRuntime`] from `world`, passes it alongside `world` to `callback`, then re-inserts it.
    ///
    /// Returns `None` if the runtime resource is missing.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaRuntime;
    /// let mut world = World::new();
    /// world.insert_non_send_resource(LuaRuntime::new());
    ///
    /// LuaRuntime::take_scope(&mut world, |runtime, world| {
    ///     runtime.scope(world).execute("x = 1")
    /// });
    /// ```
    pub fn take_scope<R>(
        world: &mut World,
        callback: impl FnOnce(&LuaRuntime, &mut World) -> R,
    ) -> Option<R> {
        // LuaRuntime is !Send and cannot be borrowed while world is also borrowed mutably,
        // so we remove it, call the closure, then re-insert it.
        let runtime = world.remove_non_send_resource::<LuaRuntime>()?;
        let result = callback(&runtime, world);
        world.insert_non_send_resource(runtime);
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
    lua: &'runtime mlua::Lua,
    _context: WorldContext<'world>,
}

impl LuaScope<'_, '_> {
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
        self.lua.load(source).exec()
    }

    /// Loads `source`, then calls `hook` passing an `EntityProxy` userdata as `self`.
    ///
    /// Resolution order (OOP-first so scripts can write `function Entity:onTick()`):
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
    /// )?;
    /// # Ok::<(), mlua::Error>(())
    /// ```
    pub fn call_hook(&self, entity: Entity, source: &str, hook: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()?;

        let globals = self.lua.globals();
        let entity_proxy = self.lua.create_userdata(EntityProxy { id: entity })?;

        // Method style takes priority so scripts can write `function Entity:onTick()`
        // using Lua's OOP convention; falling back to a plain global keeps simple
        // one-off hooks working without boilerplate.
        if let Ok(class) = globals.get::<mlua::Table>("Entity")
            && let Ok(func) = class.get::<Function>(hook)
        {
            func.call::<()>(entity_proxy)?;
            return Ok(());
        }

        if let Ok(func) = globals.get::<Function>(hook) {
            func.call::<()>(entity_proxy)?;
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
/// Construct via [`crate::LuaComponent::make_accessor`] or [`crate::serialize_component`] /
/// [`crate::deserialize_component`] / [`crate::register_component_id`] for manual wiring.
///
/// # Examples
///
/// ```rust,ignore
/// # use bevy::prelude::*;
/// # use serde::{Deserialize, Serialize};
/// # use suon_lua::{ComponentAccessor, deserialize_component, register_component_id, serialize_component};
/// #[derive(Component, Serialize, Deserialize)]
/// struct Health {
///     value: i32,
/// }
///
/// let accessor = ComponentAccessor {
///     get: serialize_component::<Health>,
///     set: deserialize_component::<Health>,
///     component_id: register_component_id::<Health>,
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
/// Components are registered automatically the first time a [`crate::LuaComponent`] is
/// inserted into any entity (via the `on_add` hook generated by `#[derive(LuaComponent)]`).
/// Manual registration via [`register_component`] or [`crate::AppLuaExt::register_lua_component`]
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
    pub(crate) components: HashMap<String, ComponentAccessor>,
    pub(crate) triggers: HashMap<String, TriggerAccessor>,
}

impl ScriptRegistry {
    /// Registers a Lua-visible component accessor under `name`.
    pub fn register_component(&mut self, name: impl Into<String>, accessor: ComponentAccessor) {
        self.components.insert(name.into(), accessor);
    }

    /// Registers a Lua-visible trigger accessor under `name`.
    pub fn register_trigger(&mut self, name: impl Into<String>, accessor: TriggerAccessor) {
        self.triggers.insert(name.into(), accessor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world
    }

    #[test]
    fn new_registers_world_global_in_lua() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        runtime
            .scope(&mut world)
            .execute("assert(world ~= nil)")
            .expect("lua exec should succeed");
    }

    #[test]
    fn exec_runs_lua_code() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
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
        let mut world = setup_world();
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
        let mut world = setup_world();
        let error = runtime
            .scope(&mut world)
            .execute("error('intentional')")
            .unwrap_err();
        assert!(error.to_string().contains("intentional"));
    }

    #[test]
    fn call_hook_entity_method_style() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();
        let scope = runtime.scope(&mut world);

        scope
            .call_hook(entity, "function Entity:onTick() ran = true end", "onTick")
            .expect("hook should execute without error");
        scope
            .execute("assert(ran == true)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_plain_function_style() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();
        let scope = runtime.scope(&mut world);

        scope
            .call_hook(entity, "function onTick(entity) ran = true end", "onTick")
            .expect("hook should execute without error");
        scope
            .execute("assert(ran == true)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_missing_hook_is_noop() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .call_hook(entity, "", "nonexistent")
            .expect("missing hook should be a noop, not an error");
    }

    #[test]
    fn call_hook_entity_method_takes_priority_over_plain_function() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();
        let scope = runtime.scope(&mut world);

        let source = "
            function Entity:onTick() style = 'method' end
            function onTick(entity)  style = 'plain'  end
        ";
        scope
            .call_hook(entity, source, "onTick")
            .expect("hook should execute without error");
        scope
            .execute("assert(style == 'method')")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_passes_entity_proxy_as_self() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();
        let scope = runtime.scope(&mut world);

        scope
            .call_hook(
                entity,
                "function Entity:onTick() received_id = self:id() end",
                "onTick",
            )
            .expect("hook should execute without error");

        scope
            .execute(&format!("assert(received_id == {})", entity.to_bits()))
            .expect("lua assertion should succeed");
    }

    #[test]
    fn exec_state_persists_across_multiple_calls_on_same_scope() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
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
        let mut world = setup_world();
        let entity = world.spawn_empty().id();
        let scope = runtime.scope(&mut world);

        let error = scope
            .call_hook(
                entity,
                "function Entity:onTick() error('hook failed') end",
                "onTick",
            )
            .unwrap_err();

        assert!(error.to_string().contains("hook failed"));
    }

    #[test]
    fn exec_empty_string_succeeds() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        runtime
            .scope(&mut world)
            .execute("")
            .expect("empty source should succeed");
    }

    #[test]
    fn globals_persist_across_separate_scope_calls_on_same_runtime() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();

        runtime.scope(&mut world).execute("x = 42").unwrap();
        runtime
            .scope(&mut world)
            .execute("assert(x == 42)")
            .expect("lua assertion should succeed");
    }

    #[test]
    fn call_hook_with_empty_source_and_no_hook_is_noop() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();

        runtime
            .scope(&mut world)
            .call_hook(entity, "", "onTick")
            .expect("empty source with no hook should be a noop");
    }

    #[test]
    fn call_hook_returns_error_when_source_has_syntax_error() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let entity = world.spawn_empty().id();

        let error = runtime
            .scope(&mut world)
            .call_hook(entity, "!! invalid !!", "onTick")
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
        let result = LuaRuntime::take_scope(&mut world, |_, _| ());
        assert!(result.is_none());
    }

    #[test]
    fn take_scope_restores_runtime_after_call() {
        let mut world = setup_world();
        world.insert_non_send_resource(LuaRuntime::new());
        LuaRuntime::take_scope(&mut world, |_, _| ());
        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn take_scope_restores_runtime_when_f_returns_err() {
        let mut world = setup_world();
        world.insert_non_send_resource(LuaRuntime::new());
        LuaRuntime::take_scope(&mut world, |_, _| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError("test".into()))
        });
        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }
}
