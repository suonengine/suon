use bevy::prelude::*;

use crate::server::{
    connection::{incoming::IncomingConnections, outgoing::OutgoingConnections},
    packet::Packet,
    system::*,
};

pub mod connection;
pub mod packet;
pub mod settings;
pub mod system;

pub(crate) struct NetworkServerPlugin;

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Packet>()
            .init_resource::<IncomingConnections>()
            .init_resource::<OutgoingConnections>()
            .add_systems(PreStartup, initialize_settings)
            .add_systems(Startup, initialize_listener)
            .add_systems(
                FixedFirst,
                (
                    cleanup_finished_connections,
                    accept_client_connections,
                    process_incoming_client_packets,
                )
                    .chain(),
            )
            .add_systems(FixedLast, flush_connection_buffers);
    }
}
