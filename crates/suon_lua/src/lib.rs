//! Lua scripting integration for the Suon MMORPG server.
//!
//! Provides a [`LuaPlugin`] that wires a [`LuaVm`] resource into the
//! application, and a [`LuaCallback`] task for invoking Lua functions
//! from the Rust event loop.
//!
//! # Lifecycle
//!
//! ```text
//! App::new()
//!     .add_plugin(LuaPlugin)
//!     .run()
//! ```
//!
//! [`LuaPlugin`] creates a [`LuaVm`], extends `package.path` to include
//! `modules/`, loads every Lua file under that directory (creating it
//! if missing), and inserts the VM into resources.  Systems and tasks
//! access it via `resources.get::<LuaVm>()`.

mod callback;
mod config;
mod plugin;
mod vm;

pub use callback::LuaCallback;
pub use config::LuaConfig;
pub use plugin::LuaPlugin;
pub use vm::LuaVm;
