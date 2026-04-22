//! Server-side networking components and schedules.

use bevy::prelude::*;

use crate::server::{
    connection::{incoming::IncomingConnections, outgoing::OutgoingConnections},
    system::*,
};

pub mod connection;
pub mod packet;
pub mod settings;
pub mod system;

pub(crate) struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        info!("Starting the server networking systems");

        app.init_resource::<IncomingConnections>()
            .init_resource::<OutgoingConnections>()
            .add_systems(PreStartup, initialize_settings)
            .add_systems(Startup, initialize_listener)
            .add_systems(
                FixedFirst,
                (cleanup_finished_connections, accept_client_connections).chain(),
            )
            .add_systems(FixedUpdate, process_incoming_client_packets)
            .add_systems(FixedLast, flush_connection_buffers);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_register_packet_message_and_connection_resources() {
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.add_plugins(NetworkServerPlugin);

        assert!(
            app.world()
                .contains_resource::<connection::incoming::IncomingConnections>(),
            "The server plugin should initialize incoming connection storage"
        );

        assert!(
            app.world()
                .contains_resource::<connection::outgoing::OutgoingConnections>(),
            "The server plugin should initialize outgoing connection storage"
        );
    }
}
