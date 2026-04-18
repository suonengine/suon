pub(crate) mod api;
pub mod commands;
pub mod lua_component;
pub mod runtime;
pub mod script;
pub(crate) mod world_cell;

pub use commands::{LuaCommands, RunLuaHook, RunLuaScript};
pub use lua_component::{
    AppLuaExt, LuaComponent, deserialize_component, register_component_id, serialize_component,
};
pub use runtime::{ComponentAccessor, LuaRuntime, LuaScope, ScriptRegistry, TriggerAccessor};
pub use script::LuaScript;

use bevy::prelude::*;

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

    use crate::{
        deserialize_component, register_component_id,
        runtime::{ComponentAccessor, LuaRuntime},
        serialize_component,
    };

    #[derive(Serialize, Deserialize, Clone)]
    struct Mana {
        points: i32,
    }

    impl Component for Mana {
        const STORAGE_TYPE: bevy::ecs::component::StorageType =
            bevy::ecs::component::StorageType::Table;
        type Mutability = bevy::ecs::component::Mutable;

        fn on_add() -> Option<bevy::ecs::lifecycle::ComponentHook> {
            Some(|mut world, _ctx| {
                if !world
                    .resource::<ScriptRegistry>()
                    .components
                    .contains_key(Mana::lua_name())
                {
                    world
                        .resource_mut::<ScriptRegistry>()
                        .register_component(Mana::lua_name(), Mana::make_accessor());
                }
            })
        }
    }

    impl LuaComponent for Mana {
        fn lua_name() -> &'static str {
            "Mana"
        }

        fn make_accessor() -> ComponentAccessor {
            ComponentAccessor {
                get: serialize_component::<Mana>,
                set: deserialize_component::<Mana>,
                component_id: register_component_id::<Mana>,
            }
        }
    }

    fn app_with_lua() -> App {
        let mut app = App::new();
        app.add_plugins(LuaPlugin);
        app
    }

    fn run_lua(app: &mut App, source: &str) {
        LuaRuntime::take_scope(app.world_mut(), |runtime, world| {
            runtime.scope(world).execute(source)
        })
        .expect("LuaRuntime missing")
        .expect("lua exec should succeed");
    }

    #[test]
    fn lua_plugin_inserts_lua_runtime() {
        let app = app_with_lua();
        assert!(app.world().get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn lua_plugin_inserts_script_registry() {
        let app = app_with_lua();
        assert!(app.world().get_resource::<ScriptRegistry>().is_some());
    }

    #[test]
    fn lua_component_auto_registers_on_first_spawn_and_is_queryable() {
        let mut app = app_with_lua();

        app.world_mut().spawn(Mana { points: 50 });

        run_lua(
            &mut app,
            "
            for id, mana in world:query('Mana'):iter() do
                assert(mana.points == 50, 'expected 50, got ' .. tostring(mana.points))
            end
        ",
        );
    }

    #[test]
    fn lua_component_auto_registers_on_first_spawn_and_is_settable() {
        let mut app = app_with_lua();

        let entity = app.world_mut().spawn(Mana { points: 0 }).id();

        run_lua(
            &mut app,
            &format!(
                "
            local entity = world:entity({bits})
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
    fn lua_exec_runs_snippet_at_next_flush() {
        let mut app = app_with_lua();

        let entity = app.world_mut().spawn(Mana { points: 10 }).id();

        app.world_mut().commands().lua_execute(format!(
            "
            local entity = world:entity({bits})
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
        let mut app = app_with_lua();
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

        app.world_mut().commands().lua_hook(entity, "onHeal");
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
        let mut app = app_with_lua();
        let entity = app.world_mut().spawn_empty().id();

        app.world_mut().commands().lua_hook(entity, "onTick");
        app.world_mut().flush();
    }

    #[test]
    fn lua_hook_is_noop_when_hook_function_is_missing() {
        let mut app = app_with_lua();
        let entity = app
            .world_mut()
            .spawn(LuaScript::new("-- no hooks here"))
            .id();

        app.world_mut().commands().lua_hook(entity, "onTick");
        app.world_mut().flush();
    }

    #[test]
    fn lua_script_source_getter_returns_stored_content() {
        let script = LuaScript::new("print('hello')");
        assert_eq!(script.source(), "print('hello')");
    }

    #[test]
    fn multiple_lua_exec_commands_queued_execute_in_order() {
        let mut app = app_with_lua();
        let entity = app.world_mut().spawn(Mana { points: 0 }).id();

        app.world_mut().commands().lua_execute(format!(
            "world:entity({bits}):set('Mana', {{ points = 1 }})",
            bits = entity.to_bits()
        ));
        app.world_mut().commands().lua_execute(format!(
            "local e = world:entity({bits})
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
    fn lua_plugin_world_global_is_accessible_in_exec() {
        let mut app = app_with_lua();
        app.world_mut()
            .commands()
            .lua_execute("assert(world ~= nil)");
        app.world_mut().flush();
    }

    #[test]
    fn lua_query_filters_entities_that_have_all_components() {
        #[derive(Serialize, Deserialize, Clone)]
        struct Stamina {
            value: i32,
        }

        impl Component for Stamina {
            const STORAGE_TYPE: bevy::ecs::component::StorageType =
                bevy::ecs::component::StorageType::Table;
            type Mutability = bevy::ecs::component::Mutable;

            fn on_add() -> Option<bevy::ecs::lifecycle::ComponentHook> {
                Some(|mut world, _ctx| {
                    if !world
                        .resource::<ScriptRegistry>()
                        .components
                        .contains_key(Stamina::lua_name())
                    {
                        world
                            .resource_mut::<ScriptRegistry>()
                            .register_component(Stamina::lua_name(), Stamina::make_accessor());
                    }
                })
            }
        }

        impl LuaComponent for Stamina {
            fn lua_name() -> &'static str {
                "Stamina"
            }

            fn make_accessor() -> ComponentAccessor {
                ComponentAccessor {
                    get: serialize_component::<Stamina>,
                    set: deserialize_component::<Stamina>,
                    component_id: register_component_id::<Stamina>,
                }
            }
        }

        let mut app = app_with_lua();

        app.world_mut().spawn(Mana { points: 1 });
        app.world_mut().spawn(Stamina { value: 2 });
        app.world_mut()
            .spawn((Mana { points: 3 }, Stamina { value: 4 }));

        run_lua(
            &mut app,
            "
            local count = 0
            for id, mana, stamina in world:query('Mana', 'Stamina'):iter() do
                count = count + 1
                assert(mana.points == 3)
                assert(stamina.value == 4)
            end
            assert(count == 1, 'expected 1, got ' .. count)
        ",
        );
    }
}
