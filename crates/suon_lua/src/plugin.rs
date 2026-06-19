use mlua::Variadic;
use suon_app::shutdown::Exit;
use tracing::{debug, error, info, warn};

use suon_app::{App, plugin::Plugin};
use suon_resource::Resources;

use crate::{DispatchError, LuaConfig, LuaVm};

/// Registers Lua scripting support into a Suon [`App`].
///
/// During [`Plugin::build`] it reads [`LuaConfig`] from resources
/// (or uses the default `"modules"` path), creates a [`LuaVm`],
/// sets up the module search path, loads every `.lua` file from the
/// configured directory (creating it if missing), and inserts the VM
/// as a resource.
///
/// # Example
///
/// ```rust,ignore
/// // Default path (modules/)
/// App::new()
///     .add_plugin(LuaPlugin)
///     .run();
///
/// // Custom path
/// App::new()
///     .add_resource(LuaConfig { modules_path: "scripts/".into(), ..Default::default() })
///     .add_plugin(LuaPlugin)
///     .run();
/// ```
#[derive(Default)]
pub struct LuaPlugin;

impl Plugin for LuaPlugin {
    fn build(&self, app: &mut App) {
        let vm = LuaVm::new();

        info!(target: "Lua", "Initialized Lua interpreter via mlua");

        let config = app
            .try_get_resource::<LuaConfig>()
            .cloned()
            .unwrap_or_default();

        let dir = config.modules_path.clone();
        let dir_str = dir.to_string_lossy().to_string();

        {
            let path_lua = format!(
                "package.path = package.path .. \";{d}/?/init.lua;{d}/?.lua\"",
                d = dir_str
            );

            vm.execute(|lua| {
                lua.load(&path_lua)
                    .set_name("path_setup")
                    .exec()
                    .expect("failed to set package.path");

                let print_fn = lua
                    .create_function(|_, args: Variadic<String>| {
                        let line = args.join("\t");
                        info!(target: "Lua", "{line}");
                        Ok(())
                    })
                    .expect("failed to create print function");

                lua.globals()
                    .set("print", print_fn)
                    .expect("failed to set print global");

                let debug_fn = lua
                    .create_function(|_, text: String| {
                        warn!(target: "Lua", "Lua Script Error: {text}");
                        Ok(())
                    })
                    .expect("failed to create debug function");

                lua.globals()
                    .set("debug", debug_fn)
                    .expect("failed to set debug global");

                if let Err(e) = crate::bindings::inject_bindings(lua) {
                    error!(target: "Lua", "Failed to register Lua bindings: {e}");
                }
            });
        }

        if !dir
            .try_exists()
            .unwrap_or_else(|e| panic!("failed to access {dir_str}: {e}"))
        {
            std::fs::create_dir_all(&dir)
                .unwrap_or_else(|e| panic!("failed to create {dir_str}: {e}"));
            info!(target: "Lua", "Created {dir_str} directory");
        }

        {
            let mut entries: Vec<_> = match std::fs::read_dir(&dir) {
                Ok(reader) => reader
                    .filter_map(|e| match e {
                        Ok(entry) => Some(entry),
                        Err(err) => {
                            warn!(target: "Lua", "Skipping unreadable directory entry in {dir_str}: {err}");
                            None
                        }
                    })
                    .collect(),
                Err(err) => {
                    error!(target: "Lua", "Could not read modules directory {dir_str}: {err}");
                    return;
                }
            };

            entries.sort_by_key(|e| e.file_name());

            for entry in &entries {
                let path = entry.path();

                if path.extension().is_some_and(|e| e == "lua") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .expect("non-utf8 module name")
                        .to_owned();

                    let result = vm.execute(|lua| {
                        lua.load(
                            std::fs::read_to_string(&path).expect("failed to read Lua module file"),
                        )
                        .set_name(&name)
                        .exec()
                    });

                    match result {
                        Ok(_) => debug!(target: "Lua", "Loaded {dir_str}/{name}.lua"),
                        Err(e) => {
                            error!(target: "Lua", "Failed to load module {dir_str}/{name}.lua: {e}")
                        }
                    }
                }
            }

            for entry in &entries {
                if !entry.path().is_dir() {
                    continue;
                }

                let init = entry.path().join("init.lua");
                if !init.is_file() {
                    continue;
                }

                let name = entry
                    .file_name()
                    .to_str()
                    .expect("non-utf8 module name")
                    .to_owned();

                let result = vm.execute(|lua| {
                    lua.load(std::fs::read_to_string(&init).expect("failed to read Lua init file"))
                        .set_name(format!("{dir_str}/{name}/init.lua"))
                        .exec()
                });

                match result {
                    Ok(_) => debug!(target: "Lua", "Loaded {dir_str}/{name}/init.lua"),
                    Err(e) => {
                        error!(target: "Lua", "Failed to load module {dir_str}/{name}/init.lua: {e}")
                    }
                }
            }
        }

        app.add_resource(vm);

        app.add_startup_system(|resources: &mut Resources| {
            let vm = resources.get::<LuaVm>();
            match vm.trigger_event("StartupEvent", ()) {
                Err(DispatchError::Cancelled) => {
                    info!(target: "Lua", "StartupEvent was cancelled, aborting startup");
                    resources.get_mut::<Exit>().trigger();
                }
                Err(e) => {
                    error!(target: "Lua", "StartupEvent error: {e}");
                }
                _ => {}
            }
        });

        app.add_shutdown_system(|resources: &mut Resources| {
            let vm = resources.get::<LuaVm>();
            if let Err(e) = vm.trigger_event("ShutdownEvent", ()) {
                error!(target: "Lua", "ShutdownEvent error: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::Mutex;
    use suon_app::{App, shutdown::Shutdown};
    use suon_channel::Channel;
    use suon_resource::Resources;

    use super::*;

    static MODULES_LOCK: Mutex<()> = Mutex::new(());

    fn with_modules_dir(callback: impl FnOnce()) {
        let _guard = MODULES_LOCK.lock();
        drop(std::fs::remove_dir_all("modules"));
        callback();
        drop(std::fs::remove_dir_all("modules"));
    }

    #[test]
    fn build_creates_resource() {
        let mut app = App::new();
        app.add_plugin(LuaPlugin);
        app.add_startup_system(|resources: &mut Resources| {
            let vm = resources.get::<LuaVm>();
            vm.execute(|lua| {
                let result: i32 = lua
                    .load("return 7 + 7")
                    .eval()
                    .expect("failed to evaluate Lua expression");

                assert_eq!(result, 14, "plugin should expose a working Lua VM");
            });

            let channel = resources.get::<Channel>();
            channel.send(Shutdown);
        });
        app.run();
    }

    #[test]
    fn build_creates_modules_dir() {
        with_modules_dir(|| {
            App::new()
                .add_plugin(LuaPlugin)
                .add_startup_system(|resources: &mut Resources| {
                    assert!(
                        std::path::Path::new("modules").exists(),
                        "modules/ directory should exist after plugin build"
                    );

                    let channel = resources.get::<Channel>();
                    channel.send(Shutdown);
                })
                .run();
        });
    }

    #[test]
    fn plugin_loads_lua_from_modules_dir() {
        with_modules_dir(|| {
            std::fs::create_dir_all("modules").expect("failed to create modules/ for test");
            std::fs::write("modules/test_module.lua", "test_global = 42")
                .expect("failed to write test Lua file");

            App::new()
                .add_plugin(LuaPlugin)
                .add_startup_system(|resources: &mut Resources| {
                    let vm = resources.get::<LuaVm>();
                    vm.execute(|lua| {
                        let value: i32 = lua.globals().get("test_global").unwrap_or(-1);
                        assert_eq!(value, 42, "plugin should load .lua files from modules/");
                    });

                    let channel = resources.get::<Channel>();
                    channel.send(Shutdown);
                })
                .run();
        });
    }
}
