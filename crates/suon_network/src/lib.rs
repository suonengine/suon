//! Networking plugins and server infrastructure for Suon.

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::server::NetworkServerPlugin;

pub mod server;

pub struct NetworkPlugins;

impl PluginGroup for NetworkPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(NetworkServerPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_add_network_plugins_to_app() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(NetworkPlugins);

        assert!(
            app.world()
                .contains_resource::<crate::server::connection::incoming::IncomingConnections>(),
            "Adding the network plugin group should initialize incoming connection state"
        );
        assert!(
            app.world()
                .contains_resource::<crate::server::connection::outgoing::OutgoingConnections>(),
            "Adding the network plugin group should initialize outgoing connection state"
        );
    }
}
