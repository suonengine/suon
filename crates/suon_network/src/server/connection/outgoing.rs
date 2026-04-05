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
    fn should_return_an_empty_list_when_no_outgoing_connections_are_queued() {
        let connections = OutgoingConnections::default();

        let read_result = connections.read();
        assert!(
            read_result.is_empty(),
            "Reading from an empty channel should return no queued connections"
        );
    }

    #[test]
    fn should_queue_and_drain_outgoing_connections() {
        let connections = OutgoingConnections::default();
        let queued_connection = (
            Entity::from_bits(99),
            "127.0.0.1:7172"
                .parse()
                .expect("The test socket address should parse"),
        );

        connections
            .send(queued_connection)
            .expect("The queue should accept outgoing connections");

        let queued = connections.read();

        assert_eq!(
            queued,
            vec![queued_connection],
            "read should return the queued outgoing connection in insertion order"
        );
        assert!(
            connections.read().is_empty(),
            "Reading again should return an empty list after the queue is drained"
        );
    }
}
