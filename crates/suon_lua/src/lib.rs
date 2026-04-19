//! Lua scripting for Bevy — attach scripts to entities, query the ECS from Lua, and fire triggers.
//!
//! # Architecture
//!
//! ```text
//! LuaPlugin
//!   ├── LuaRuntime (NonSend resource) — owns the mlua::Lua VM
//!   ├── ScriptRegistry (Resource)     — component/trigger accessors keyed by name
//!   └── world_cell                    — thread-local raw pointer bridging Rust ↔ Lua callbacks
//! ```
//!
//! # Quick start
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use serde::{Deserialize, Serialize};
//! use suon_lua::{LuaPlugin, LuaCommands, LuaScript};
//!
//! #[derive(LuaComponent, Serialize, Deserialize)]
//! struct Health { value: i32 }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins((DefaultPlugins, LuaPlugin))
//!         .add_systems(Startup, |mut commands: Commands| {
//!             commands.spawn((
//!                 Health { value: 100 },
//!                 LuaScript::new("function Entity:onTick()
//!                     local hp = self:get('Health')
//!                     self:set('Health', { value = hp.value - 1 })
//!                 end"),
//!             ));
//!         })
//!         .add_systems(Update, |mut commands: Commands, q: Query<Entity>| {
//!             for entity in &q { commands.lua_execute(format!("-- tick {}", entity.to_bits())); }
//!         })
//!         .run();
//! }
//! ```

extern crate self as suon_lua;

pub(crate) mod api;
pub mod commands;
pub mod lua_component;
pub mod runtime;
pub mod script;
pub(crate) mod world_cell;

pub use commands::{Hook, LuaCommands, RunLuaHook, RunLuaScript};
pub use lua_component::{AppLuaExt, LuaComponent, WorldLuaComponentExt};
pub use runtime::{
    ComponentAccessor, LuaRuntime, LuaScope, ScriptRegistry, TriggerAccessor, WorldLuaRuntimeExt,
};
pub use script::LuaScript;

use bevy::prelude::*;

/// Bevy plugin that sets up the Lua scripting runtime.
///
/// Inserts [`LuaRuntime`] as a non-send resource and initialises [`ScriptRegistry`].
/// Register components with [`AppLuaExt::register_lua_component`] or rely on the
/// automatic registration that fires on the first `insert` of a [`LuaComponent`].
///
/// # Examples
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use suon_lua::LuaPlugin;
///
/// App::new()
///     .add_plugins((MinimalPlugins, LuaPlugin))
///     .run();
/// ```
pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptRegistry>()
            .insert_non_send_resource(LuaRuntime::new());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use suon_macros::{LuaComponent, LuaHook};

    use crate::runtime::{ComponentAccessor, LuaRuntime};

    #[derive(LuaHook, Serialize)]
    #[lua(name = "onTick")]
    struct TickHook;

    #[derive(LuaHook, Serialize)]
    #[lua(name = "onHeal")]
    struct HealHook;

    #[derive(LuaComponent, Serialize, Deserialize, Clone)]
    struct Mana {
        points: i32,
    }

    fn run_lua(app: &mut App, source: &str) {
        let result = app
            .world_mut()
            .lua_runtime(|runtime, world| runtime.scope(world).execute(source));
        let result = result.expect("LuaRuntime missing");
        result.expect("lua exec should succeed");
    }

    #[test]
    fn lua_plugin_inserts_lua_runtime() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        assert!(app.world().get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn lua_plugin_inserts_script_registry() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        assert!(app.world().get_resource::<ScriptRegistry>().is_some());
    }

    #[test]
    fn lua_component_auto_registers_on_first_spawn_and_is_queryable() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        app.world_mut().spawn(Mana { points: 50 });

        run_lua(
            &mut app,
            "
            for id, mana in Query('Mana'):iter() do
                assert(mana.points == 50, 'expected 50, got ' .. tostring(mana.points))
            end
        ",
        );
    }

    #[test]
    fn lua_component_auto_registers_on_first_spawn_and_is_settable() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        let entity = app.world_mut().spawn(Mana { points: 0 }).id();

        run_lua(
            &mut app,
            &format!(
                "
            local entity = Entity({bits})
            entity:set('Mana', {{ points = 99 }})
        ",
                bits = entity.to_bits()
            ),
        );

        assert_eq!(
            app.world()
                .get::<Mana>(entity)
                .expect("Mana should be present")
                .points,
            99
        );
    }

    #[test]
    fn lua_execute_runs_snippet_at_next_flush() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        let entity = app.world_mut().spawn(Mana { points: 10 }).id();

        app.world_mut().commands().lua_execute(format!(
            "
            local entity = Entity({bits})
            entity:set('Mana', {{ points = 7 }})
        ",
            bits = entity.to_bits()
        ));
        app.world_mut().flush();

        assert_eq!(
            app.world()
                .get::<Mana>(entity)
                .expect("Mana should be present")
                .points,
            7
        );
    }

    #[test]
    fn lua_hook_calls_entity_method_from_script() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        let entity = app
            .world_mut()
            .spawn((
                Mana { points: 0 },
                LuaScript::new(
                    "function Entity:onHeal()
                        local mana = self:get('Mana')
                        self:set('Mana', { points = mana.points + 10 })
                    end",
                ),
            ))
            .id();

        app.world_mut()
            .commands()
            .lua_hook(entity, HealHook)
            .expect("hook args should serialize");
        app.world_mut().flush();

        assert_eq!(
            app.world()
                .get::<Mana>(entity)
                .expect("Mana should be present")
                .points,
            10
        );
    }

    #[test]
    fn lua_hook_is_noop_when_entity_lacks_script() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        let entity = app.world_mut().spawn_empty().id();

        app.world_mut()
            .commands()
            .lua_hook(entity, TickHook)
            .expect("hook args should serialize");
        app.world_mut().flush();
    }

    #[test]
    fn lua_hook_is_noop_when_hook_function_is_missing() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        let entity = app
            .world_mut()
            .spawn(LuaScript::new("-- no hooks here"))
            .id();

        app.world_mut()
            .commands()
            .lua_hook(entity, TickHook)
            .expect("hook args should serialize");
        app.world_mut().flush();
    }

    #[test]
    fn lua_hook_is_noop_when_entity_is_despawned_before_flush() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        let entity = app
            .world_mut()
            .spawn(LuaScript::new("function Entity:onTick() ran = true end"))
            .id();

        app.world_mut()
            .commands()
            .lua_hook(entity, TickHook)
            .expect("hook args should serialize");
        app.world_mut().despawn(entity);
        app.world_mut().flush();

        assert!(app.world().get_entity(entity).is_err());
    }

    #[test]
    fn lua_script_source_getter_returns_stored_content() {
        let script = LuaScript::new("print('hello')");
        assert_eq!(script.source(), "print('hello')");
    }

    #[test]
    fn multiple_lua_execute_commands_queued_execute_in_order() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        let entity = app.world_mut().spawn(Mana { points: 0 }).id();

        app.world_mut().commands().lua_execute(format!(
            "Entity({bits}):set('Mana', {{ points = 1 }})",
            bits = entity.to_bits()
        ));
        app.world_mut().commands().lua_execute(format!(
            "local e = Entity({bits})
             local m = e:get('Mana')
             e:set('Mana', {{ points = m.points + 10 }})",
            bits = entity.to_bits()
        ));
        app.world_mut().flush();

        assert_eq!(
            app.world()
                .get::<Mana>(entity)
                .expect("Mana should exist")
                .points,
            11
        );
    }

    #[test]
    fn lua_plugin_entity_global_is_accessible_in_exec() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        app.world_mut()
            .commands()
            .lua_execute("assert(Entity ~= nil)");
        app.world_mut().flush();
    }

    #[test]
    fn on_add_does_not_overwrite_existing_accessor() {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        // First spawn triggers auto-registration via the on_add hook.
        app.world_mut().spawn(Mana { points: 0 });

        // Replace with a sentinel that always returns points = 999.
        app.world_mut()
            .resource_mut::<ScriptRegistry>()
            .components
            .insert(
                "Mana".to_string(),
                ComponentAccessor {
                    get: |_entity, _world| Some(serde_json::json!({ "points": 999 })),
                    set: |_, _, _| {},
                    component_id: |world| world.register_component::<Mana>(),
                },
            );

        // Second spawn — on_add should skip because "Mana" is already in the registry.
        app.world_mut().spawn(Mana { points: 50 });

        // Call the accessor: if it still returns 999 the sentinel wasn't overwritten.
        let get_fn = app
            .world()
            .resource::<ScriptRegistry>()
            .components
            .get("Mana")
            .expect("Mana should still be in registry")
            .get;

        let mut dummy_world = World::new();
        let dummy_entity = dummy_world.spawn_empty().id();
        assert_eq!(
            get_fn(dummy_entity, &mut dummy_world),
            Some(serde_json::json!({ "points": 999 })),
            "on_add should not overwrite an existing accessor"
        );
    }

    #[test]
    fn change_detection_fires_after_lua_set() {
        // Verifies that entity:set() triggers Bevy's change detection by checking
        // the component's change tick after the Lua call.
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.insert_non_send_resource(LuaRuntime::new());

        #[derive(LuaComponent, Serialize, Deserialize, Clone)]
        struct Coins {
            count: i32,
        }

        let entity = world.spawn(Coins { count: 0 }).id();
        // Advance the world tick so the spawn's change stamp is in the past.
        world.clear_trackers();
        world.increment_change_tick();

        let result = world.lua_runtime(|runtime, w| {
            runtime.scope(w).execute(&format!(
                "Entity({}):set('Coins', {{ count = 5 }})",
                entity.to_bits()
            ))
        });
        let result = result.expect("LuaRuntime missing");
        result.expect("lua exec should succeed");

        // ComponentTicks::is_changed uses world.last_change_tick() as last_run, so
        // the insert that happened after increment_change_tick() must be detected.
        let ticks = world
            .entity(entity)
            .get_change_ticks::<Coins>()
            .expect("Coins should have change ticks");

        assert!(
            ticks.is_changed(world.last_change_tick(), world.change_tick()),
            "Coins should be detected as changed after Lua entity:set()"
        );
    }

    #[test]
    fn lua_hook_runs_each_update_when_queued_by_system() {
        use bevy::app::Update;

        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        let entity = app
            .world_mut()
            .spawn((
                Mana { points: 0 },
                LuaScript::new(
                    "function Entity:onTick()
                        local m = self:get('Mana')
                        self:set('Mana', { points = m.points + 1 })
                    end",
                ),
            ))
            .id();

        app.add_systems(Update, move |mut commands: Commands| {
            commands
                .lua_hook(entity, TickHook)
                .expect("hook args should serialize");
        });

        app.update();
        app.update();

        assert_eq!(
            app.world()
                .get::<Mana>(entity)
                .expect("Mana should exist")
                .points,
            2,
            "hook should have incremented Mana.points once per Update tick"
        );
    }

    #[test]
    fn lua_query_filters_entities_that_have_all_components() {
        #[derive(LuaComponent, Serialize, Deserialize, Clone)]
        struct Stamina {
            value: i32,
        }

        let mut app = App::new();
        app.add_plugins(LuaPlugin);

        app.world_mut().spawn(Mana { points: 1 });
        app.world_mut().spawn(Stamina { value: 2 });
        app.world_mut()
            .spawn((Mana { points: 3 }, Stamina { value: 4 }));

        run_lua(
            &mut app,
            "
            local count = 0
            for id, mana, stamina in Query('Mana', 'Stamina'):iter() do
                count = count + 1
                assert(mana.points == 3)
                assert(stamina.value == 4)
            end
            assert(count == 1, 'expected 1, got ' .. count)
        ",
        );
    }
}
