use mlua::Function;
use suon_channel::TaskHandler;
use suon_resource::Resources;

use crate::LuaVm;

/// A task that invokes a Lua function previously registered with
/// [`LuaVm::store`].
///
/// Sent through the [`Channel`] from Lua code.  When executed, the
/// task looks up the function by its numeric handle in the Lua
/// registry, calls it with no arguments, and cleans up.
///
/// [`Channel`]: suon_channel::Channel
/// [`LuaVm::store`]: crate::LuaVm::store
pub struct LuaCallback {
    /// Numeric handle returned by [`LuaVm::store`].
    pub id: u64,
}

impl TaskHandler for LuaCallback {
    fn run(self: Box<Self>, resources: &mut Resources) {
        let vm = resources.get::<LuaVm>();
        vm.execute(|lua| {
            let key = format!("_lua_fn_{}", self.id);
            if let Ok(func) = lua.named_registry_value::<Function>(&key)
                && let Err(error) = func.call::<()>(())
            {
                eprintln!("[Lua] callback {id} error: {error}", id = self.id);
            }

            if let Err(error) = lua.unset_named_registry_value(&key) {
                eprintln!(
                    "[Lua] failed to unregister callback {id}: {error}",
                    id = self.id
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Error;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[test]
    fn callback_calls_stored_function() {
        let mut resources = Resources::default();
        resources.insert(LuaVm::new());

        let called = Arc::new(AtomicBool::new(false));
        let flag = called.clone();

        let id = {
            let vm_ref = resources.get::<LuaVm>();
            vm_ref.execute(|lua| {
                let func = lua
                    .create_function(move |_, ()| {
                        flag.store(true, Ordering::SeqCst);
                        Ok(())
                    })
                    .expect("failed to create test function");

                vm_ref.store(func).expect("failed to store test function")
            })
        };

        let task = Box::new(LuaCallback { id });
        task.run(&mut resources);

        assert!(
            called.load(Ordering::SeqCst),
            "callback should have executed the stored Lua function"
        );
    }

    #[test]
    fn callback_missing_id_does_not_panic() {
        let mut resources = Resources::default();
        resources.insert(LuaVm::new());

        let task = Box::new(LuaCallback { id: 999 });
        task.run(&mut resources);
    }

    #[test]
    fn callback_cleans_up_after_run() {
        let mut resources = Resources::default();
        resources.insert(LuaVm::new());

        let id = {
            let vm_ref = resources.get::<LuaVm>();
            vm_ref.execute(|lua| {
                let func = lua
                    .create_function(|_, ()| -> Result<(), Error> { Ok(()) })
                    .expect("failed to create test function");

                vm_ref.store(func).expect("failed to store test function")
            })
        };

        let task = Box::new(LuaCallback { id });
        task.run(&mut resources);

        let vm_ref = resources.get::<LuaVm>();
        let result = vm_ref.restore(id);
        assert!(
            result.is_err(),
            "callback should remove the function from the registry after execution"
        );
    }
}
