//! Movement systems for stepping and teleporting entities across the Suon world.
//!
//! This crate owns intent-driven movement behaviors that update
//! [`suon_position::prelude::Position`], optionally update
//! [`suon_position::prelude::Floor`], and record previous values for the axes
//! that changed so downstream crates can react consistently to movement.
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
//! use suon_chunk::prelude::*;
//! use suon_movement::prelude::*;
//! use suon_position::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(ChunkPlugins);
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

/// Common movement types and plugin groups for downstream crates.
pub mod prelude {
    pub use super::{
        MovementPlugins,
        step::{Step, StepAcrossChunk, StepError, StepIntent, StepRejected, path::StepPath},
        teleport::{
            Teleport, TeleportAcrossChunk, TeleportError, TeleportIntent, TeleportRejected,
        },
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
        use crate::prelude::*;
        use suon_position::prelude::*;

        let _ = std::mem::size_of::<MovementPlugins>();
        let _ = std::mem::size_of::<Step>();
        let _ = std::mem::size_of::<StepAcrossChunk>();
        let _ = std::mem::size_of::<StepError>();
        let _ = std::mem::size_of::<StepIntent>();
        let _ = std::mem::size_of::<StepPath>();
        let _ = std::mem::size_of::<StepRejected>();
        let _ = std::mem::size_of::<Teleport>();
        let _ = std::mem::size_of::<TeleportAcrossChunk>();
        let _ = std::mem::size_of::<TeleportError>();
        let _ = std::mem::size_of::<TeleportIntent>();
        let _ = std::mem::size_of::<TeleportRejected>();

        assert_eq!(
            Direction::North.offset(),
            (0, 1),
            "The prelude should expose step direction helpers"
        );
    }
}
