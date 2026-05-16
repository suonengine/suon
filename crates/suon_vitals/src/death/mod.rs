//! Death and revival flows.

pub(crate) mod die;
pub(crate) mod revive;

use bevy::{app::PluginGroupBuilder, prelude::*};

pub(crate) struct DeathPlugins;

impl PluginGroup for DeathPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(die::DiePlugin)
            .add(revive::RevivePlugin)
    }
}

pub(crate) mod prelude {
    pub use super::{die::Die, revive::Revive};
}
