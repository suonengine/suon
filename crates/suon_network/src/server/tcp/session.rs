use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::connection::id::ConnectionId;

use crate::{connection::manager::ConnectionManager, server::shutdown::Shutdown};

/// Coordinates the lifecycle of a single TCP connection.
///
/// Owns the reader and writer task handles so that a failure in one
/// can be propagated to the other, and provides a single `close()`
/// entry point that triggers graceful shutdown for both halves.
#[allow(dead_code)]
pub(crate) struct ConnectionSession {
    pub id: ConnectionId,
    reader: JoinHandle<()>,
    writer: JoinHandle<()>,
    connection_manager: Arc<ConnectionManager>,
    shutdown: Shutdown,
}

impl ConnectionSession {
    pub fn new(
        id: ConnectionId,
        reader: JoinHandle<()>,
        writer: JoinHandle<()>,
        connection_manager: Arc<ConnectionManager>,
        shutdown: Shutdown,
    ) -> Self {
        ConnectionSession {
            id,
            reader,
            writer,
            connection_manager,
            shutdown,
        }
    }

    /// Waits for either the reader or the writer to finish, then
    /// triggers shutdown for the other half and cleans up the
    /// connection registry.
    pub async fn run(self) {
        tokio::select! {
            _ = self.reader => {},
            _ = self.writer => {},
        }

        self.connection_manager.unregister(self.id);
        self.shutdown.trigger();
    }
}
