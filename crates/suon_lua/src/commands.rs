//! Bevy [`Command`]s that queue Lua execution until the next command flush.
//!
//! Use [`LuaCommands`] on [`Commands`] to enqueue snippets or typed hook calls
//! without taking direct ownership of [`crate::LuaRuntime`].

use bevy::prelude::*;
use serde::Serialize;
use serde_json::Value as Json;
use std::sync::Arc;

use crate::{runtime::WorldLuaRuntimeExt, script::LuaScript};

/// Typed hook description sent from Rust to Lua.
///
/// The hook name comes from [`Hook::name`]. The hook arguments come from the
/// serialized value:
/// - structs become positional arguments using their field values
/// - tuples and arrays become positional arguments
/// - a single scalar becomes one positional argument
/// - unit/empty hooks become no extra arguments
///
/// This lets hooks be declared ergonomically on the Rust side:
///
/// ```rust,ignore
/// use serde::Serialize;
/// use suon_macros::LuaHook;
///
/// #[derive(LuaHook, Serialize)]
/// struct Move {
///     from: (i32, i32),
///     to: (i32, i32),
/// }
/// ```
pub trait Hook: Serialize {
    /// Lua-visible method name, e.g. `onMove`.
    fn name() -> &'static str;

    /// Converts the hook payload into the JSON shape consumed by the Lua runtime.
    ///
    /// Default behavior:
    /// - objects become arrays of their field values
    /// - arrays stay arrays
    /// - scalars stay scalars
    /// - `null` stays `null`
    fn into_args(self) -> serde_json::Result<Json>
    where
        Self: Sized,
    {
        Ok(match serde_json::to_value(self)? {
            Json::Object(object) => {
                Json::Array(object.into_iter().map(|(_, value)| value).collect())
            }
            value => value,
        })
    }
}

/// Bevy [`Command`] that calls a named hook on an entity's [`LuaScript`].
///
/// Enqueued by [`LuaCommands::lua_hook`]; applied at the next command flush.
/// If the entity has no [`LuaScript`], or the script does not define the named
/// hook, the command is a silent no-op. Errors from the hook are logged and
/// swallowed so one bad script cannot stall the command queue.
pub struct RunLuaHook {
    /// Target entity whose attached [`LuaScript`] should receive the hook call.
    pub(crate) entity: Entity,

    /// Lua-visible hook name, such as `onMove`.
    pub(crate) hook: &'static str,

    /// Serialized positional arguments passed after `self` in Lua.
    pub(crate) args: Json,
}

impl Command for RunLuaHook {
    /// Executes the queued hook call against the current Bevy world.
    fn apply(self, world: &mut World) {
        let Some(source) = world
            .get::<LuaScript>(self.entity)
            .map(|script| script.source().to_owned())
        else {
            return;
        };

        let result = world.lua_runtime(|runtime, world| {
            runtime
                .scope(world)
                .call_hook(self.entity, &source, self.hook, self.args)
        });

        if let Some(Err(error)) = result {
            bevy::log::error!("lua hook '{}' on {:?}: {error}", self.hook, self.entity);
        }
    }
}

/// Bevy [`Command`] that executes a Lua snippet at the next command flush.
///
/// Enqueued by [`LuaCommands::lua_execute`]. Errors are logged and swallowed.
pub struct RunLuaScript {
    /// Lua source code scheduled for execution.
    pub(crate) source: Arc<str>,
}

impl Command for RunLuaScript {
    /// Executes the queued Lua source against the shared runtime.
    fn apply(self, world: &mut World) {
        let result = world.lua_runtime(|runtime, world| runtime.scope(world).execute(&self.source));

        if let Some(Err(error)) = result {
            bevy::log::error!("lua_execute error: {error}");
        }
    }
}

/// Extends [`Commands`] with Lua execution methods.
pub trait LuaCommands {
    /// Queues a typed hook call on `entity`'s [`LuaScript`].
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use bevy::prelude::*;
    /// # use serde::Serialize;
    /// # use suon_lua::LuaCommands;
    /// # use suon_macros::LuaHook;
    /// #[derive(LuaHook, Serialize)]
    /// struct Damage {
    ///     amount: i32,
    /// }
    ///
    /// fn on_damage(mut commands: Commands, entity: Entity) {
    ///     let result = commands.lua_hook(entity, Damage { amount: 10 });
    ///     assert!(result.is_ok());
    /// }
    /// ```
    fn lua_hook<H: Hook>(&mut self, entity: Entity, hook: H) -> serde_json::Result<()>;

    /// Queues a [`RunLuaScript`] command that executes `source` at the next flush.
    fn lua_execute(&mut self, source: impl Into<Arc<str>>);
}

impl LuaCommands for Commands<'_, '_> {
    /// Queues a typed hook invocation to run on the next command flush.
    fn lua_hook<H: Hook>(&mut self, entity: Entity, hook: H) -> serde_json::Result<()> {
        self.queue(RunLuaHook {
            entity,
            hook: H::name(),
            args: hook.into_args()?,
        });
        Ok(())
    }

    /// Queues raw Lua source to run on the next command flush.
    fn lua_execute(&mut self, source: impl Into<Arc<str>>) {
        self.queue(RunLuaScript {
            source: source.into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LuaRuntime, runtime::ScriptRegistry, script::LuaScript};
    use suon_macros::LuaHook;

    #[derive(LuaHook, Serialize)]
    #[lua(name = "onTick")]
    struct TickHook;

    #[derive(LuaHook, Serialize)]
    struct Move {
        x: i32,
        y: i32,
    }

    #[derive(LuaHook, Serialize)]
    struct Damage(i32, &'static str);

    #[test]
    fn run_lua_hook_executes_hook_on_entity_with_script() {
        let mut world = World::new();
        world.init_resource::<ScriptRegistry>();
        world.insert_non_send_resource(LuaRuntime::new());

        let entity = world
            .spawn(LuaScript::new("function Entity:onTick() ran = true end"))
            .id();

        RunLuaHook {
            entity,
            hook: TickHook::name(),
            args: Json::Null,
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| runtime.scope(world).execute("assert(ran == true)"))
            .expect("LuaRuntime missing")
            .expect("lua assertion failed");
    }

    #[test]
    fn run_lua_hook_is_noop_when_entity_has_no_script() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn_empty().id();

        RunLuaHook {
            entity,
            hook: TickHook::name(),
            args: Json::Null,
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_hook_is_noop_when_hook_function_is_missing() {
        let mut world = World::new();

        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world.spawn(LuaScript::new("-- no hooks")).id();

        RunLuaHook {
            entity,
            hook: TickHook::name(),
            args: Json::Null,
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_hook_logs_error_and_does_not_panic_on_runtime_error() {
        let mut world = World::new();

        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world
            .spawn(LuaScript::new("function Entity:onTick() error('boom') end"))
            .id();

        RunLuaHook {
            entity,
            hook: TickHook::name(),
            args: Json::Null,
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_script_executes_snippet() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        RunLuaScript {
            source: "counter = 42".into(),
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| runtime.scope(world).execute("assert(counter == 42)"))
            .expect("LuaRuntime missing")
            .expect("lua assertion failed");
    }

    #[test]
    fn run_lua_script_logs_error_and_does_not_panic_on_syntax_error() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        RunLuaScript {
            source: "!!! not lua !!!".into(),
        }
        .apply(&mut world);
    }

    #[test]
    fn run_lua_script_logs_error_and_does_not_panic_on_runtime_error() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        RunLuaScript {
            source: "error('boom')".into(),
        }
        .apply(&mut world);
    }

    #[test]
    fn multiple_run_lua_script_commands_execute_in_order() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        RunLuaScript {
            source: "order = 'first'".into(),
        }
        .apply(&mut world);

        RunLuaScript {
            source: "order = order .. '_second'".into(),
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| {
                runtime
                    .scope(world)
                    .execute("assert(order == 'first_second')")
                    .expect("LuaRuntime missing")
            })
            .expect("lua assertion failed");
    }

    #[test]
    fn error_in_run_lua_script_does_not_prevent_next_command() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        RunLuaScript {
            source: "error('intentional')".into(),
        }
        .apply(&mut world);

        RunLuaScript {
            source: "after_error = true".into(),
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| {
                runtime
                    .scope(world)
                    .execute("assert(after_error == true)")
                    .expect("LuaRuntime missing")
            })
            .expect("lua assertion failed");
    }

    #[test]
    fn error_in_run_lua_hook_does_not_prevent_next_command() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world
            .spawn(LuaScript::new("function Entity:onTick() error('boom') end"))
            .id();

        RunLuaHook {
            entity,
            hook: TickHook::name(),
            args: Json::Null,
        }
        .apply(&mut world);

        RunLuaScript {
            source: "after_hook_error = true".into(),
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| {
                runtime
                    .scope(world)
                    .execute("assert(after_hook_error == true)")
                    .expect("LuaRuntime missing")
            })
            .expect("lua assertion failed");
    }

    #[test]
    fn run_lua_hook_passes_struct_fields_as_positional_arguments() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world
            .spawn(LuaScript::new(
                "function Entity:onMove(x, y) seen_x = x; seen_y = y end",
            ))
            .id();

        RunLuaHook {
            entity,
            hook: Move::name(),
            args: Move { x: 3, y: 7 }
                .into_args()
                .expect("hook args should serialize"),
        }
        .apply(&mut world);

        world
            .lua_runtime(|runtime, world| {
                runtime
                    .scope(world)
                    .execute("assert(seen_x == 3 and seen_y == 7)")
            })
            .expect("LuaRuntime missing")
            .expect("lua assertion failed");
    }

    #[test]
    fn lua_hook_uses_typed_hook_name_and_arguments() {
        let mut world = World::new();
        world.insert_non_send_resource(LuaRuntime::new());
        world.init_resource::<ScriptRegistry>();

        let entity = world
            .spawn(LuaScript::new(
                "function Entity:onDamage(amount, source) total = amount; who = source end",
            ))
            .id();

        world
            .commands()
            .lua_hook(entity, Damage(12, "lava"))
            .expect("hook args should serialize");

        world.flush();

        world
            .lua_runtime(|runtime, world| {
                runtime
                    .scope(world)
                    .execute("assert(total == 12 and who == 'lava')")
            })
            .expect("LuaRuntime missing")
            .expect("lua assertion failed");
    }
}
