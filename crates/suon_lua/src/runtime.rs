use bevy::{ecs::component::ComponentId, prelude::*};
use mlua::Function;
use serde_json::Value as Json;
use std::collections::HashMap;

use crate::{api::entity::EntityProxy, world_cell::WorldContext};

/// Non-Send Bevy resource that owns the Lua state.
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
    pub fn scope<'runtime, 'world>(
        &'runtime self,
        world: &'world mut World,
    ) -> LuaScope<'runtime, 'world> {
        LuaScope {
            lua: &self.lua,
            _context: WorldContext::enter(world),
        }
    }
}

/// Execution context that combines an active Lua state with exclusive world access.
///
/// Created via [`LuaRuntime::scope`]. Clears the world pointer on drop.
pub struct LuaScope<'runtime, 'world> {
    lua: &'runtime mlua::Lua,
    _context: WorldContext<'world>,
}

impl LuaScope<'_, '_> {
    /// Executes a Lua source snippet.
    pub fn exec(&self, source: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()
    }

    /// Loads `source`, then calls `hook` passing an [`EntityProxy`] as `self`.
    ///
    /// Looks up `Entity:<hook>` first (method style), then a plain global `<hook>`.
    pub fn call_hook(&self, entity: Entity, source: &str, hook: &str) -> mlua::Result<()> {
        self.lua.load(source).exec()?;

        let globals = self.lua.globals();
        let entity_proxy = self.lua.create_userdata(EntityProxy { id: entity })?;

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

/// Type-erased component accessor registered for use from Lua.
pub struct ComponentAccessor {
    /// Serializes the component to JSON. Returns `None` if the entity lacks the component.
    pub get: fn(Entity, &mut World) -> Option<Json>,
    /// Deserializes JSON and inserts/updates the component on the entity.
    pub set: fn(Entity, &mut World, Json),
    /// Returns the [`ComponentId`] for this component type, registering it if needed.
    pub component_id: fn(&mut World) -> ComponentId,
}

/// Type-erased trigger registered for use from `entity:trigger(Name, {...})`.
pub struct TriggerAccessor {
    /// Deserializes the args table and fires the trigger on the entity.
    pub fire: fn(Entity, &mut World, Json),
}

#[cfg(test)]
impl LuaScope<'_, '_> {
    pub(crate) fn eval<T: mlua::FromLua>(&self, expression: &str) -> mlua::Result<T> {
        self.lua.load(expression).eval::<T>()
    }
}

/// Registry of components and triggers exposed to Lua scripts.
#[derive(Resource, Default)]
pub struct ScriptRegistry {
    pub(crate) components: HashMap<String, ComponentAccessor>,
    pub(crate) triggers: HashMap<String, TriggerAccessor>,
}

impl ScriptRegistry {
    pub fn register_component(&mut self, name: impl Into<String>, accessor: ComponentAccessor) {
        self.components.insert(name.into(), accessor);
    }

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
            .exec("assert(world ~= nil)")
            .expect("lua exec should succeed");
    }

    #[test]
    fn exec_runs_lua_code() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let scope = runtime.scope(&mut world);
        scope
            .exec("result = 1 + 2")
            .expect("lua exec should succeed");
        let result: i64 = scope.eval("result").expect("eval should return integer");
        assert_eq!(result, 3);
    }

    #[test]
    fn exec_returns_error_on_syntax_error() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let error = runtime
            .scope(&mut world)
            .exec("this is !! not lua")
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
            .exec("error('intentional')")
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

        assert!(
            scope
                .eval::<bool>("ran == true")
                .expect("eval should return bool")
        );
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

        assert!(
            scope
                .eval::<bool>("ran == true")
                .expect("eval should return bool")
        );
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

        assert_eq!(
            scope
                .eval::<String>("style")
                .expect("eval should return string"),
            "method"
        );
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

        let received_id: i64 = scope
            .eval("received_id")
            .expect("eval should return integer");
        assert_eq!(received_id, entity.to_bits() as i64);
    }

    #[test]
    fn exec_state_persists_across_multiple_calls_on_same_scope() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();
        let scope = runtime.scope(&mut world);

        scope.exec("counter = 0").expect("lua exec should succeed");
        scope
            .exec("counter = counter + 1")
            .expect("lua exec should succeed");
        scope
            .exec("counter = counter + 1")
            .expect("lua exec should succeed");

        assert_eq!(
            scope
                .eval::<i64>("counter")
                .expect("eval should return integer"),
            2
        );
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
            .exec("")
            .expect("empty source should succeed");
    }

    #[test]
    fn globals_persist_across_separate_scope_calls_on_same_runtime() {
        let runtime = LuaRuntime::new();
        let mut world = setup_world();

        runtime.scope(&mut world).exec("x = 42").unwrap();
        let val: i64 = runtime.scope(&mut world).eval("x").unwrap();
        assert_eq!(val, 42);
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
}
