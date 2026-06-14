use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU64, Ordering},
    thread::{self, ThreadId},
};

use mlua::{Error, Function, IntoLuaMulti, Lua, ObjectLike, Table, Value};
use suon_macros::Resource;
use tracing::{debug, error, info, warn};

use crate::error::DispatchError;

/// Resource that owns the Lua scripting virtual machine.
///
/// # Thread safety
///
/// `LuaVm` wraps `mlua::Lua` which is `!Send + !Sync` because the Lua C API
/// uses thread-local state internally.  Access to the underlying `Lua` value
/// is restricted to the thread that created the VM (the "owner" thread) via
/// a runtime `debug_assert!`.  This is sound because the application's event
/// loop processes all tasks sequentially on a single thread.
#[derive(Resource)]
pub struct LuaVm {
    lua: UnsafeCell<Lua>,
    owner: ThreadId,
    next_id: AtomicU64,
}

// SAFETY: All access to `self.lua` is gated by a runtime thread-ownership
// check (debug builds).  The application architecture guarantees that only
// the single event-loop thread touches the Lua state.
unsafe impl Send for LuaVm {}
unsafe impl Sync for LuaVm {}

impl Default for LuaVm {
    fn default() -> Self {
        Self::new()
    }
}

impl LuaVm {
    /// Creates a new Lua VM with the standard library loaded.
    ///
    /// # Panics
    ///
    /// Panics if the underlying `mlua::Lua` state cannot be initialized.
    pub fn new() -> Self {
        info!(target: "Lua", "Creating Lua VM");
        LuaVm {
            lua: UnsafeCell::new(Lua::new()),
            owner: thread::current().id(),
            next_id: AtomicU64::new(1),
        }
    }

    fn assert_owner(&self) {
        debug_assert_eq!(
            thread::current().id(),
            self.owner,
            "LuaVm accessed from a different thread than the one that created it"
        );
    }

    /// Provides scoped access to the underlying `Lua` state.
    pub fn execute<R>(&self, callback: impl FnOnce(&Lua) -> R) -> R {
        self.assert_owner();

        // SAFETY: We are on the owner thread; no other thread holds a
        // reference to the inner `Lua`.
        callback(unsafe { &*self.lua.get() })
    }

    /// Calls `EventClass:trigger(arguments...)` on the global `EventClass`.
    ///
    /// Errors are logged via `tracing`; returns [`DispatchError`] on failure.
    pub fn trigger_event(&self, name: &str, args: impl IntoLuaMulti) -> Result<(), DispatchError> {
        self.assert_owner();

        let lua = unsafe { &*self.lua.get() };
        let class: Table = lua.globals().get(name).map_err(|_| {
            warn!(target: "Lua", "Event class {name} not found");
            DispatchError::NoHandler
        })?;

        let result: Value = class.call_method("trigger", args).map_err(|e| {
            error!(target: "Lua", "Event {name} error: {e}");
            DispatchError::HandlerError
        })?;

        match result {
            Value::Boolean(true) => Ok(()),
            Value::Boolean(false) => {
                debug!(target: "Lua", "Event {name} was cancelled");
                Err(DispatchError::Cancelled)
            }
            other => {
                warn!(
                    target: "Lua",
                    "Event {name} returned non-boolean value ({other:?}), treating as cancelled"
                );
                Err(DispatchError::NoResult)
            }
        }
    }

    /// Store a Lua function and return its numeric handle.
    pub fn store(&self, func: Function) -> Result<u64, Error> {
        self.assert_owner();

        let lua = unsafe { &*self.lua.get() };
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let key = format!("_lua_fn_{id}");
        lua.set_named_registry_value(&key, func)?;
        debug!(target: "Lua", "Stored callback function as id={id}");
        Ok(id)
    }

    /// Retrieves a previously stored Lua function by its handle.
    pub fn restore(&self, id: u64) -> Result<Function, Error> {
        self.assert_owner();

        let lua = unsafe { &*self.lua.get() };
        let key = format!("_lua_fn_{id}");
        let func = lua.named_registry_value(&key)?;
        debug!(target: "Lua", "Restored callback function id={id}");
        Ok(func)
    }

    /// Removes a previously stored Lua function from the registry.
    pub fn remove(&self, id: u64) -> Result<(), Error> {
        self.assert_owner();

        let lua = unsafe { &*self.lua.get() };
        let key = format!("_lua_fn_{id}");
        lua.unset_named_registry_value(&key)?;
        debug!(target: "Lua", "Removed callback function id={id}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_vm() {
        let vm = LuaVm::new();
        vm.execute(|lua| {
            let result: i32 = lua
                .load("return 1 + 2")
                .eval()
                .expect("failed to evaluate Lua expression in test");
            assert_eq!(result, 3, "vm should evaluate Lua expressions");
        });
    }

    #[test]
    fn execute_provides_scoped_access() {
        let vm = LuaVm::new();
        let sum = vm.execute(|lua| {
            lua.load("return 10 * 5")
                .eval::<i32>()
                .expect("failed to evaluate multiplication in test")
        });
        assert_eq!(sum, 50, "execute should return the closure result");
    }

    #[test]
    fn store_restore_roundtrip() {
        let vm = LuaVm::new();
        vm.execute(|lua| {
            let func = lua
                .create_function(|_, ()| -> Result<(), Error> { Ok(()) })
                .expect("failed to create no-op function");

            let id = vm
                .store(func)
                .expect("failed to store function in registry");

            let restored = vm.restore(id).expect("failed to restore function by id");

            restored
                .call::<()>(())
                .expect("restored function should execute without error");
        });
    }

    #[test]
    fn store_increments_ids() {
        let vm = LuaVm::new();
        vm.execute(|lua| {
            let callback = lua
                .create_function(|_, ()| -> Result<(), Error> { Ok(()) })
                .expect("failed to create first function");

            let first_id = vm.store(callback).expect("failed to store first function");

            let callback = lua
                .create_function(|_, ()| -> Result<(), Error> { Ok(()) })
                .expect("failed to create second function");

            let second_id = vm.store(callback).expect("failed to store second function");
            assert!(second_id > first_id, "each store should return a higher id");
        });
    }

    #[test]
    fn restore_missing_returns_error() {
        let vm = LuaVm::new();
        let result = vm.restore(999);
        assert!(result.is_err(), "restoring a nonexistent id should fail");
    }

    #[test]
    fn remove_cleans_up() {
        let vm = LuaVm::new();
        vm.execute(|lua| {
            let func = lua
                .create_function(|_, ()| -> Result<(), Error> { Ok(()) })
                .expect("failed to create no-op function");
            let id = vm.store(func).expect("failed to store function");

            vm.remove(id).expect("remove should succeed on a stored id");

            let result = vm.restore(id);
            assert!(result.is_err(), "after remove the function should be gone");
        });
    }

    #[test]
    fn remove_nonexistent_succeeds() {
        let vm = LuaVm::new();
        let result = vm.remove(999);
        assert!(result.is_ok(), "removing a nonexistent id should not error");
    }

    #[test]
    fn trigger_event_ok_on_true() {
        let vm = LuaVm::new();

        let _ = vm.execute(|lua| {
            let class = lua.create_table()?;
            let trigger = lua.create_function(|_, value: bool| Ok(value))?;
            class.set("trigger", trigger)?;
            lua.globals().set("TestEvent", class)?;
            Ok::<(), mlua::Error>(())
        });

        assert!(vm.trigger_event("TestEvent", (true,)).is_ok());
    }

    #[test]
    fn trigger_event_cancelled_on_false() {
        let vm = LuaVm::new();

        let _ = vm.execute(|lua| {
            let class = lua.create_table()?;
            let trigger = lua.create_function(|_, ()| Ok(false))?;
            class.set("trigger", trigger)?;
            lua.globals().set("TestEvent", class)?;
            Ok::<(), mlua::Error>(())
        });

        let result = vm.trigger_event("TestEvent", ());
        assert!(
            matches!(&result, Err(DispatchError::Cancelled)),
            "expected Cancelled, got {result:?}"
        );
    }

    #[test]
    fn trigger_event_no_result_on_nil() {
        let vm = LuaVm::new();

        let _ = vm.execute(|lua| {
            let class = lua.create_table()?;
            let trigger = lua.create_function(|_, ()| Ok(mlua::Value::Nil))?;
            class.set("trigger", trigger)?;
            lua.globals().set("TestEvent", class)?;
            Ok::<(), mlua::Error>(())
        });

        let result = vm.trigger_event("TestEvent", ());
        assert!(
            matches!(&result, Err(DispatchError::NoResult)),
            "expected NoResult, got {result:?}"
        );
    }

    #[test]
    fn trigger_event_missing_class_returns_error() {
        let vm = LuaVm::new();
        let result = vm.trigger_event("MissingEvent", ());
        assert!(matches!(result, Err(DispatchError::NoHandler)));
    }
}
