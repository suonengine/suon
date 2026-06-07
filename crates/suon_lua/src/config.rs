use std::path::PathBuf;

use suon_macros::Resource;

/// Configuration for [`LuaPlugin`](crate::LuaPlugin).
///
/// Register this resource **before** `LuaPlugin` to customise where Lua
/// scripts are loaded from:
///
/// ```rust,ignore
/// App::new()
///     .add_resource(LuaConfig { modules_path: "scripts/".into(), ..default() })
///     .add_plugin(LuaPlugin)
///     .run();
/// ```
///
/// When not provided, `LuaPlugin` uses `"modules"` as the default path.
#[derive(Resource, Clone)]
pub struct LuaConfig {
    /// Directory scanned for `.lua` files at startup.
    ///
    /// Defaults to `"modules"`.
    pub modules_path: PathBuf,
}

impl Default for LuaConfig {
    fn default() -> Self {
        LuaConfig {
            modules_path: PathBuf::from("modules"),
        }
    }
}
