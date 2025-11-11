use bevy::prelude::*;

use crate::server::connection::{Connection, outgoing::OutgoingConnections};

/// Processes and removes any finished connections.
pub(crate) fn cleanup_finished_connections(
    mut commands: Commands,
    outgoing_connections: Res<OutgoingConnections>,
    query: Query<&Connection>,
) {
    for (client, addr) in outgoing_connections.read() {
        if let Ok(connection) = query.get(client) {
            // Remove the connection if the address matches.
            if connection.addr() == addr {
                commands.entity(client).remove::<Connection>();

                info!("Removed outgoing connection for {addr} (client {client}).");
            } else {
                warn!(
                    "Address mismatch for client {client}: expected {addr}, found {}. Skipping \
                     removal.",
                    connection.addr()
                );
            }
        } else {
            debug!("No active connection found for {addr} (client {client}). Skipping cleanup.");
        }
    }
}
