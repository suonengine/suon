//! Umbrella crate for the Suon engine workspace.
//!
//! `suon` acts as the publishable entry point for crates.io by re-exporting the
//! core workspace crates and exposing a single [`SuonPlugin`] that installs the
//! main Bevy plugins maintained by this workspace.
//!
//! # Quick start
//! ```no_run
//! use bevy::prelude::*;
//! use suon::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(SuonPlugin);
//! ```

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use std::time::Duration;
use suon_chunk::ChunkPlugin;
use suon_lua::LuaPlugin;
use suon_movement::prelude::MovementPlugins;
use suon_network::NetworkPlugins;

pub use settings::Settings;
pub use suon_chunk;
pub use suon_lua;
pub use suon_movement;
pub use suon_network;
pub use suon_observability::{self, ObservabilityPlugin, ObservabilitySettings};
pub use suon_position;
pub use suon_task;

mod settings;

/// Common imports for apps that build on top of the umbrella `suon` crate.
pub mod prelude {
    pub use crate::{ObservabilityPlugin, ObservabilitySettings, Settings, SuonPlugin};
    pub use suon_chunk::{Chunk, ChunkPlugin, chunks::Chunks, content::AtChunk};
    pub use suon_lua::{AppLuaExt, LuaCommands, LuaComponent, LuaPlugin, LuaScript};
    pub use suon_movement::prelude::*;
    pub use suon_network::NetworkPlugins;
    pub use suon_position::{
        direction::Direction, floor::Floor, position::Position, previous_floor::PreviousFloor,
        previous_position::PreviousPosition,
    };
}

/// Main plugin that wires together the core Suon runtime crates.
///
/// This plugin bootstraps the Bevy app from `Settings.toml`, installs
/// headless runtime plugins, and wires together the main Suon domain plugins.
pub struct SuonPlugin;

impl Plugin for SuonPlugin {
    fn build(&self, app: &mut App) {
        let settings = Settings::load_or_default().expect("Failed to load Suon settings.");

        let minimal_plugins = MinimalPlugins.set(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions::with_num_threads(settings.threads),
        });

        let minimal_plugins = if settings.schedule_runner {
            minimal_plugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                settings.event_loop,
            )))
        } else {
            minimal_plugins
        };

        app.add_plugins(minimal_plugins);
        app.insert_resource(settings);
        app.insert_resource(Time::<Fixed>::from_seconds(settings.fixed_event_loop));

        app.add_plugins((
            ObservabilityPlugin,
            ChunkPlugin,
            MovementPlugins,
            NetworkPlugins,
        ));
        app.add_plugins(LuaPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_suon_plugin_zero_sized() {
        assert_eq!(std::mem::size_of::<SuonPlugin>(), 0);
    }
}
