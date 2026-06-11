use mlua::{Error, Function, IntoLuaMulti, Lua, ObjectLike, Table, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use suon_macros::Resource;

/// Resource that owns the Lua scripting virtual machine.
///
/// Inserted into [`Resources`](suon_resource::Resources) by
/// [`LuaPlugin`](crate::LuaPlugin).  Access it from systems or task
/// handlers via `resources.get::<LuaVm>()`.
///
/// # Thread safety
///
/// `Lua` is not `Send` or `Sync`.  All systems and tasks in Suon
/// run on the single thread that called [`App::run`], so the VM is
/// never accessed concurrently.  The `Send + Sync + Resource` impl is
/// safe under this guarantee.
///
/// [`App::run`]: suon_app::App::run
#[derive(Resource)]
pub struct LuaVm {
    lua: Lua,
    next_id: AtomicU64,
}

impl Default for LuaVm {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for LuaVm {}
unsafe impl Sync for LuaVm {}

impl LuaVm {
    /// Creates a new Lua VM with the standard library loaded.
    pub fn new() -> Self {
        LuaVm {
            lua: Lua::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// Provides scoped access to the underlying `Lua` state.
    ///
    /// ```rust,ignore
    /// vm.execute(|lua| {
    ///     lua.load("print('hello')").exec()
    /// });
    /// ```
    pub fn execute<R>(&self, callback: impl FnOnce(&Lua) -> R) -> R {
        callback(&self.lua)
    }

    /// Dispatches a named event to Lua by calling
    pub fn dispatch(&self, name: &str, args: impl IntoLuaMulti) -> bool {
        match self.lua.globals().get::<Table>("Events") {
            Ok(events) => match events.call_method::<Value>("trigger", (name, args)) {
                Ok(result) => result.as_boolean().unwrap_or(true),
                Err(error) => {
                    eprintln!("[Lua] dispatch error for {name}: {error}");
                    false
                }
            },
            Err(error) => {
                eprintln!("[Lua] Events global not found: {error}");
                false
            }
        }
    }

    /// Stores a Lua function in the registry and returns a numeric handle.
    pub fn store(&self, func: Function) -> Result<u64, Error> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let key = format!("_lua_fn_{id}");
        self.lua.set_named_registry_value(&key, func)?;
        Ok(id)
    }

    /// Retrieves a previously stored Lua function by its handle.
    pub fn restore(&self, id: u64) -> Result<Function, Error> {
        let key = format!("_lua_fn_{id}");
        self.lua.named_registry_value(&key)
    }

    /// Removes a previously stored Lua function from the registry.
    pub fn remove(&self, id: u64) -> Result<(), Error> {
        let key = format!("_lua_fn_{id}");
        self.lua.unset_named_registry_value(&key)
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
    fn dispatch_returns_false_when_no_events_global() {
        let vm = LuaVm::new();
        let result = vm.dispatch("TestEvent", (42,));
        assert!(!result, "dispatch should return false without Events table");
    }

    #[test]
    fn dispatch_evaluates_trivial_events_table() {
        let vm = LuaVm::new();

        vm.execute(|lua| {
            let events = lua.create_table().expect("failed to create Events table");
            let trigger = lua
                .create_function(|_, ()| Ok(true))
                .expect("failed to create trigger function");

            events
                .set("trigger", trigger)
                .expect("failed to set Events.trigger");

            lua.globals()
                .set("Events", events)
                .expect("failed to set global Events");
        });

        let result = vm.dispatch("TestEvent", ());
        assert!(result, "dispatch should return the Events.trigger result");
    }
}
