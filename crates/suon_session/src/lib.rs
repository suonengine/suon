mod component;
mod protocol;

pub mod prelude {
    pub use crate::{
        component::{Player, Session},
        SessionPlugins,
    };
}

pub struct SessionPlugins;

impl bevy::app::PluginGroup for SessionPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(protocol::SessionProtocolPlugin)
    }
}
