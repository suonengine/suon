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
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_chunk::{Chunk, ChunkPlugin, chunks::Chunks};
//! use suon_movement::prelude::{MovementPlugins, StepIntent};
//! use suon_position::{direction::Direction, floor::Floor, position::Position};
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(ChunkPlugin);
//! app.add_plugins(MovementPlugins);
//!
//! let chunk = app.world_mut().spawn(Chunk).id();
//! app.insert_resource(Chunks::from_iter([
//!     (Position { x: 1, y: 1 }, chunk),
//!     (Position { x: 2, y: 1 }, chunk),
//! ]));
//!
//! let entity = app.world_mut().spawn((Position { x: 1, y: 1 }, Floor { z: 0 })).id();
//! app.world_mut().trigger(StepIntent {
//!     to: Direction::East,
//!     entity,
//! });
//! app.update();
//!
//! assert_eq!(
//!     *app.world().get::<Position>(entity).unwrap(),
//!     Position { x: 2, y: 1 }
//! );
//! ```

use bevy::{app::PluginGroupBuilder, prelude::*};

mod step;
mod teleport;

pub mod prelude {
    pub use super::{
        MovementPlugins,
        step::{Step, StepAcrossChunk, StepIntent, path::StepPath},
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

    #[test]
    fn should_allow_prelude_imports_for_public_movement_types() {
        use crate::prelude::{
            MovementPlugins, Step, StepAcrossChunk, StepIntent, StepPath, Teleport,
            TeleportAcrossChunk, TeleportIntent,
        };
        use suon_position::direction::Direction;

        let _ = std::mem::size_of::<MovementPlugins>();
        let _ = std::mem::size_of::<Step>();
        let _ = std::mem::size_of::<StepAcrossChunk>();
        let _ = std::mem::size_of::<StepIntent>();
        let _ = std::mem::size_of::<StepPath>();
        let _ = std::mem::size_of::<Teleport>();
        let _ = std::mem::size_of::<TeleportAcrossChunk>();
        let _ = std::mem::size_of::<TeleportIntent>();

        assert_eq!(
            Direction::North.offset(),
            (0, 1),
            "The prelude should expose step direction helpers"
        );
    }
}
