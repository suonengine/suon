//! Bevy [`Command`]s that queue Lua execution until the next command flush.
//!
//! Use [`LuaCommands`] on [`Commands`] to enqueue snippets or hook calls without
//! taking direct ownership of [`LuaRuntime`].

use bevy::prelude::*;
use std::sync::Arc;

use crate::{runtime::LuaRuntime, script::LuaScript};

/// Bevy [`Command`] that calls a named hook on an entity's [`LuaScript`].
///
/// Enqueued by [`LuaCommands::lua_hook`]; applied at the next command flush.
/// If the entity has no [`LuaScript`], or the script does not define the named
/// hook, the command is a silent no-op. Errors from the hook are logged and
/// swallowed so one bad script cannot stall the command queue.
///
/// # Examples
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use suon_lua::{LuaCommands, LuaScript};
///
/// fn queue_hook(mut commands: Commands, entity: Entity) {
///     commands.entity(entity).insert(LuaScript::new(
///         "function Entity:onSpawned() print('spawned') end",
///     ));
///     commands.lua_hook(entity, "onSpawned");
/// }
/// ```
pub struct RunLuaHook {
    pub(crate) entity: Entity,
    pub(crate) hook: &'static str,
}

impl Command for RunLuaHook {
    fn apply(self, world: &mut World) {
        let Some(source) = world
            .get::<LuaScript>(self.entity)
            .map(LuaScript::shared_source)
        else {
            return;
        };

        let result = LuaRuntime::take_scope(world, |runtime, world| {
            runtime
                .scope(world)
                .call_hook(self.entity, &source, self.hook)
        });

        if let Some(Err(error)) = result {
            bevy::log::error!("lua hook '{}' on {:?}: {error}", self.hook, self.entity);
        }
    }
}

/// Bevy [`Command`] that executes a Lua snippet at the next command flush.
///
/// Enqueued by [`LuaCommands::lua_execute`]. Errors are logged and swallowed.
///
/// # Examples
///
/// ```rust,ignore
/// use bevy::prelude::*;
/// use suon_lua::LuaCommands;
///
/// fn queue_script(mut commands: Commands) {
///     commands.lua_execute("counter = (counter or 0) + 1");
/// }
/// ```
pub struct RunLuaScript {
    pub(crate) source: Arc<str>,
}

impl Command for RunLuaScript {
    fn apply(self, world: &mut World) {
        let result = LuaRuntime::take_scope(world, |runtime, world| {
            runtime.scope(world).execute(&self.source)
        });

        if let Some(Err(error)) = result {
            bevy::log::error!("lua_execute error: {error}");
        }
    }
}

/// Extends [`Commands`] with Lua execution methods.
pub trait LuaCommands {
    /// Queues a [`RunLuaHook`] command that calls `hook` on `entity`'s [`LuaScript`].
    ///
    /// The hook runs at the next [`Commands`] flush, inside the shared Lua VM.
    /// No-op if the entity has no script or the hook is not defined.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaCommands;
    /// fn on_damage(mut commands: Commands, entity: Entity) {
    ///     commands.lua_hook(entity, "onDamage");
    /// }
    /// ```
    fn lua_hook(&mut self, entity: Entity, hook: &'static str);

    /// Queues a [`RunLuaScript`] command that executes `source` at the next flush.
    ///
    /// The snippet runs inside the shared Lua VM and has access to the ECS globals.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use suon_lua::LuaCommands;
    /// fn reset_all(mut commands: Commands) {
    ///     commands.lua_execute(
    ///         "for id, hp in Query('Health'):iter() do hp.value = 100 end",
    ///     );
    /// }
    /// ```
    fn lua_execute(&mut self, source: impl Into<Arc<str>>);
}

impl LuaCommands for Commands<'_, '_> {
    fn lua_hook(&mut self, entity: Entity, hook: &'static str) {
        self.queue(RunLuaHook { entity, hook });
    }

    fn lua_execute(&mut self, source: impl Into<Arc<str>>) {
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

    #[test]
    fn run_lua_hook_executes_hook_on_entity_with_script() {
        let mut world = setup_world();
        let entity = world
            .spawn(LuaScript::new("function Entity:onTick() ran = true end"))
            .id();

        RunLuaHook {
            entity,
            hook: "onTick",
        }
        .apply(&mut world);

        LuaRuntime::take_scope(&mut world, |runtime, world| {
            runtime.scope(world).execute("assert(ran == true)")
        })
        .expect("LuaRuntime missing")
        .expect("lua assertion failed");
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
            .spawn(LuaScript::new("function Entity:onTick() error('boom') end"))
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

        LuaRuntime::take_scope(&mut world, |runtime, world| {
            runtime.scope(world).execute("assert(counter == 42)")
        })
        .expect("LuaRuntime missing")
        .expect("lua assertion failed");
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

        LuaRuntime::take_scope(&mut world, |runtime, world| {
            runtime
                .scope(world)
                .execute("assert(order == 'first_second')")
        })
        .expect("LuaRuntime missing")
        .expect("lua assertion failed");
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

        LuaRuntime::take_scope(&mut world, |runtime, world| {
            runtime.scope(world).execute("assert(after_error == true)")
        })
        .expect("LuaRuntime missing")
        .expect("lua assertion failed");
    }

    #[test]
    fn error_in_run_lua_hook_does_not_prevent_next_command() {
        let mut world = setup_world();
        let entity = world
            .spawn(LuaScript::new("function Entity:onTick() error('boom') end"))
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

        LuaRuntime::take_scope(&mut world, |runtime, world| {
            runtime
                .scope(world)
                .execute("assert(after_hook_error == true)")
        })
        .expect("LuaRuntime missing")
        .expect("lua assertion failed");
    }
}
