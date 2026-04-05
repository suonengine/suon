//! Movement systems for stepping and teleporting entities across the Suon world.
//!
//! This crate owns intent-driven movement behaviors that update
//! [`suon_position::position::Position`] and record
//! [`suon_position::previous_position::PreviousPosition`] so downstream crates can
//! react consistently to movement.
//!
//! # Modules
//!
//! - Step flow via [`prelude::Step`], [`prelude::StepIntent`], and
//!   [`prelude::StepPath`]
//! - Teleport flow via [`prelude::Teleport`] and [`prelude::TeleportIntent`]

use bevy::{app::PluginGroupBuilder, prelude::*};

mod step;
mod teleport;

pub mod prelude {
    pub use super::{
        MovementPlugins,
        step::{Step, StepAcrossChunk, StepIntent, direction::StepDirection, path::StepPath},
        teleport::{Teleport, TeleportAcrossChunk, TeleportIntent},
    };

    pub(crate) use super::step::timer::StepTimer;
}

pub struct MovementPlugins;

impl PluginGroup for MovementPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(step::StepPlugin)
            .add(teleport::TeleportPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_build_movement_plugin_group() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(MovementPlugins);
        app.update();

        assert!(
            std::mem::size_of::<MovementPlugins>() == 0,
            "The movement plugin group should remain a zero-sized configuration marker"
        );
    }
}
