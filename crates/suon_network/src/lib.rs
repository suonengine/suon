use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::server::NetworkServerPlugin;

pub mod server;

pub struct NetworkPlugins;

impl PluginGroup for NetworkPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(NetworkServerPlugin)
    }
}
