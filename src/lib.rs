//! Umbrella crate for the Suon engine workspace.
//!
//! `suon` acts as the publishable entry point for crates.io by re-exporting the
//! core workspace crates and exposing a single [`SuonPlugin`] that installs the
//! main Bevy plugins maintained by this workspace.
//!
//! # Quick start
//! ```no_run
//! use suon::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(SuonPlugin);
//! ```

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use std::time::Duration;
use suon_chunk::prelude::*;
use suon_lua::prelude::*;
use suon_market::prelude::*;
use suon_movement::prelude::*;
use suon_network::prelude::*;
use suon_rng::prelude::*;

mod settings;
mod system;

/// Common imports for apps that build on top of the umbrella `suon` crate.
pub mod prelude {
    pub use crate::{SuonPlugin, settings::Settings};
    pub use bevy::prelude::*;
    pub use suon_checksum::prelude::*;
    pub use suon_chunk::prelude::*;
    pub use suon_database::prelude::*;
    pub use suon_lua::prelude::*;
    pub use suon_macros::*;
    pub use suon_market::prelude::*;
    pub use suon_movement::prelude::*;
    pub use suon_network::prelude::*;
    pub use suon_observability::prelude::*;
    pub use suon_position::prelude::*;
    pub use suon_protocol::prelude::*;
    pub use suon_serde::prelude::*;
    pub use suon_task::prelude::*;
    pub use suon_xtea::prelude::*;
}

/// Main plugin that wires together the core Suon runtime crates.
///
/// This plugin bootstraps the Bevy app from `Settings.toml`, installs
/// headless runtime plugins, and wires together the main Suon domain plugins.
pub struct SuonPlugin;

impl Plugin for SuonPlugin {
    fn build(&self, app: &mut App) {
        let settings = system::bootstrap_settings(app);

        let minimal_plugins = MinimalPlugins.set(TaskPoolPlugin {
            task_pool_options: TaskPoolOptions::with_num_threads(settings.threads),
        });

        let minimal_plugins = if settings.schedule_runner {
            minimal_plugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                settings.event_loop_seconds(),
            )))
        } else {
            minimal_plugins
        };

        app.add_plugins(minimal_plugins)
            .add_systems(PreStartup, system::initialize_settings)
            .add_systems(
                Startup,
                (system::initialize_fixed_time, system::log_startup_summary).chain(),
            )
            .add_plugins((
                suon_observability::ObservabilityPlugin,
                ChunkPlugins,
                MovementPlugins,
                MarketPlugins,
                NetworkPlugins,
                RNGPlugin,
            ))
            .add_plugins(LuaPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_keep_suon_plugin_zero_sized() {
        assert_eq!(std::mem::size_of::<SuonPlugin>(), 0);
    }

    #[test]
    fn should_expose_bevy_and_suon_api_through_prelude() {
        use crate::prelude::*;

        struct PreludeTable;
        impl Table for PreludeTable {}

        let _ = std::mem::size_of::<Adler32Checksum>();
        let _ = std::mem::size_of::<App>();
        let _ = std::mem::size_of::<Chunks>();
        let _ = std::mem::size_of::<Commands<'static, 'static>>();
        let _ = std::mem::size_of::<DbMut<'static, PreludeTable>>();
        let _ = std::mem::size_of::<Encoder>();
        let _ = std::mem::size_of::<Position>();
        let _ = std::mem::size_of::<DbSettings>();
        let _ = std::mem::size_of::<SuonPlugin>();
        let _ = std::mem::size_of::<XTEAKey>();
    }
}
