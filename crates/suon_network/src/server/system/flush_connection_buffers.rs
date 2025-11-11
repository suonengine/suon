use bevy::prelude::*;

use crate::server::connection::Connection;

/// Flushes the outgoing packet buffers of all active client connections.
///
/// This function iterates over all entities with a `Connection` component and attempts
/// to send any buffered outgoing data.
pub(crate) fn flush_connection_buffers(query: Query<(Entity, &Connection)>) {
    for (client, connection) in query {
        // Attempt to flush the buffer for this connection
        if let Some(flushed_bytes) = connection.flush() {
            debug!(
                "Flushed {} bytes from outgoing buffer of client {} (entity {:?})",
                flushed_bytes,
                connection.addr(),
                client
            );

            trace!(
                "Connection flush completed for client {} (entity {:?})",
                connection.addr(),
                client
            );
        }
    }
}
