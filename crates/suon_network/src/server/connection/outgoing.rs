use bevy::prelude::*;

/// Represents a newly accepted client connection.
pub(crate) type OutgoingConnection = (Entity, std::net::SocketAddr);

/// Manages outgoing client connections waiting to be processed.
#[derive(Resource, Clone)]
pub(crate) struct OutgoingConnections {
    /// Channel sender used to enqueue new outgoing connections.
    sender: crossbeam_channel::Sender<OutgoingConnection>,

    /// Channel receiver used to dequeue connections for processing.
    receiver: crossbeam_channel::Receiver<OutgoingConnection>,
}

impl Default for OutgoingConnections {
    /// Creates a new `OutgoingConnections` instance with an unbounded channel.
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<OutgoingConnection>();
        Self { sender, receiver }
    }
}

impl OutgoingConnections {
    /// Enqueues a new outgoing connection for processing.
    pub fn send(
        &self,
        connection: OutgoingConnection,
    ) -> Result<(), crossbeam_channel::SendError<OutgoingConnection>> {
        self.sender.send(connection)
    }

    /// Retrieves all currently queued outgoing connections without blocking.
    pub fn read(&self) -> Vec<OutgoingConnection> {
        self.receiver
            .try_iter()
            .collect::<Vec<OutgoingConnection>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_read_empty_channel_returns_error() {
        // Create a new resource with no connections
        let connections = OutgoingConnections::default();

        // Attempt to read from an empty channel
        let read_result = connections.read();
        assert!(
            read_result.is_empty(),
            "Reading from an empty channel should return an error"
        );
    }
}
