use bevy::prelude::*;

use crate::{runtime::LuaRuntime, script::LuaScript};

/// Bevy [`Command`] that calls a named hook on an entity's [`LuaScript`].
///
/// The hook is resolved as `Entity:<hook>(self)` first, then as a plain global `<hook>(entity)`.
pub struct RunLuaHook {
    pub(crate) entity: Entity,
    pub(crate) hook: &'static str,
}

impl Command for RunLuaHook {
    fn apply(self, world: &mut World) {
        let Some(source) = world
            .get::<LuaScript>(self.entity)
            .map(|script| script.source().to_owned())
        else {
            return;
        };

        let result = with_runtime(world, |runtime, world| {
            runtime
                .scope(world)
                .call_hook(self.entity, &source, self.hook)
        });

        if let Some(Err(error)) = result {
            bevy::log::error!("lua hook '{}' on {:?}: {error}", self.hook, self.entity);
        }
    }
}

/// Bevy [`Command`] that executes a Lua snippet directly.
pub struct RunLuaScript {
    pub(crate) source: String,
}

impl Command for RunLuaScript {
    fn apply(self, world: &mut World) {
        let result = with_runtime(world, |runtime, world| {
            runtime.scope(world).exec(&self.source)
        });

        if let Some(Err(error)) = result {
            bevy::log::error!("lua_exec error: {error}");
        }
    }
}

/// Removes [`LuaRuntime`] from `world`, passes it alongside `world` to `f`, then re-inserts it.
///
/// Returns `None` if the runtime resource is missing.
fn with_runtime<R>(world: &mut World, f: impl FnOnce(&LuaRuntime, &mut World) -> R) -> Option<R> {
    let runtime = world.remove_non_send_resource::<LuaRuntime>()?;
    let result = f(&runtime, world);
    world.insert_non_send_resource(runtime);
    Some(result)
}

/// Extends [`Commands`] with Lua execution methods.
pub trait LuaCommands {
    /// Queues a named hook call on an entity's [`LuaScript`].
    fn lua_hook(&mut self, entity: Entity, hook: &'static str);
    /// Queues execution of an arbitrary Lua snippet at the next command flush.
    fn lua_exec(&mut self, source: impl Into<String>);
}

impl LuaCommands for Commands<'_, '_> {
    fn lua_hook(&mut self, entity: Entity, hook: &'static str) {
        self.queue(RunLuaHook { entity, hook });
    }

    fn lua_exec(&mut self, source: impl Into<String>) {
        self.queue(RunLuaScript {
            source: source.into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{runtime::ScriptRegistry, script::LuaScript};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();
        world
    }

    fn read_lua_global<T: mlua::FromLua>(world: &mut World, expression: &str) -> T {
        with_runtime(world, |runtime, world| {
            runtime.scope(world).eval::<T>(expression)
        })
        .expect("LuaRuntime missing")
        .expect("eval failed")
    }

    #[test]
    fn run_lua_hook_executes_hook_on_entity_with_script() {
        let mut world = setup_world();
        let entity = world
            .spawn(LuaScript::new("function onTick(entity) ran = true end"))
            .id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);

        assert!(read_lua_global::<bool>(&mut world, "ran == true"));
    }

    #[test]
    fn run_lua_hook_is_noop_when_entity_has_no_script() {
        let mut world = setup_world();
        let entity = world.spawn_empty().id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_hook_is_noop_when_hook_function_is_missing() {
        let mut world = setup_world();
        let entity = world.spawn(LuaScript::new("-- no hooks")).id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_hook_logs_error_and_does_not_panic_on_runtime_error() {
        let mut world = setup_world();
        let entity = world
            .spawn(LuaScript::new("function onTick(entity) error('boom') end"))
            .id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_script_executes_snippet() {
        let mut world = setup_world();

        RunLuaScript {
            source: "counter = 42".into(),
        }
        .apply(&mut world);

        assert_eq!(read_lua_global::<i64>(&mut world, "counter"), 42);
    }

    #[test]
    fn run_lua_script_logs_error_and_does_not_panic_on_syntax_error() {
        let mut world = setup_world();

        RunLuaScript {
            source: "!!! not lua !!!".into(),
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_script_logs_error_and_does_not_panic_on_runtime_error() {
        let mut world = setup_world();

        RunLuaScript {
            source: "error('boom')".into(),
        }
        .apply(&mut world);
    }

    #[test]
    fn with_runtime_returns_none_when_runtime_is_missing() {
        let mut world = World::new();
        let result = with_runtime(&mut world, |_, _| ());
        assert!(result.is_none());
    }

    #[test]
    fn with_runtime_restores_resource_after_successful_call() {
        let mut world = setup_world();
        with_runtime(&mut world, |_, _| ());
        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn with_runtime_restores_resource_even_when_f_returns_error() {
        let mut world = setup_world();

        with_runtime(&mut world, |_, _| -> mlua::Result<()> {
            Err(mlua::Error::RuntimeError("test".into()))
        });

        assert!(world.get_non_send_resource::<LuaRuntime>().is_some());
    }

    #[test]
    fn multiple_run_lua_script_commands_execute_in_order() {
        let mut world = setup_world();

        RunLuaScript {
            source: "order = 'first'".into(),
        }
        .apply(&mut world);
        RunLuaScript {
            source: "order = order .. '_second'".into(),
        }
        .apply(&mut world);

        assert_eq!(
            read_lua_global::<String>(&mut world, "order"),
            "first_second"
        );
    }

    #[test]
    fn error_in_run_lua_script_does_not_prevent_next_command() {
        let mut world = setup_world();

        RunLuaScript {
            source: "error('intentional')".into(),
        }
        .apply(&mut world);
        RunLuaScript {
            source: "after_error = true".into(),
        }
        .apply(&mut world);

        assert!(read_lua_global::<bool>(&mut world, "after_error == true"));
    }

    #[test]
    fn error_in_run_lua_hook_does_not_prevent_next_command() {
        let mut world = setup_world();
        let entity = world
            .spawn(LuaScript::new("function onTick(e) error('boom') end"))
            .id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);
        RunLuaScript {
            source: "after_hook_error = true".into(),
        }
        .apply(&mut world);

        assert!(read_lua_global::<bool>(
            &mut world,
            "after_hook_error == true"
        ));
    }
}
