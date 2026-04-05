//! Movement systems for stepping and teleporting entities across the Suon world.
//!
//! This crate owns intent-driven movement behaviors that update
//! [`suon_position::position::Position`] and record
//! [`suon_position::previous_position::PreviousPosition`] so downstream crates can
//! react consistently to movement.
//!
//! # Modules
//!
//! - [`step`]: single-tile movement, path advancement, and step events
//! - [`teleport`]: direct relocation intents and cross-chunk teleport events

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
