use bevy::prelude::*;
use smol::net::TcpStream;

/// Manages incoming client connections that are waiting to be processed.
#[derive(Resource, Clone)]
pub(crate) struct IncomingConnections {
    /// Channel sender used to enqueue new incoming connections.
    sender: crossbeam_channel::Sender<TcpStream>,

    /// Channel receiver used to dequeue connections for processing.
    receiver: crossbeam_channel::Receiver<TcpStream>,
}

impl Default for IncomingConnections {
    /// Creates a new `IncomingConnections` instance with an unbounded channel.
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<TcpStream>();
        Self { sender, receiver }
    }
}

impl IncomingConnections {
    /// Enqueues a new incoming connection for processing.
    pub fn send(
        &self,
        connection: TcpStream,
    ) -> Result<(), crossbeam_channel::SendError<TcpStream>> {
        self.sender.send(connection)
    }

    /// Retrieves all currently queued incoming connections without blocking.
    pub fn read(&self) -> Vec<TcpStream> {
        self.receiver.try_iter().collect::<Vec<TcpStream>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_an_empty_list_when_no_incoming_connections_are_queued() {
        let connections = IncomingConnections::default();

        let read_result = connections.read();
        assert!(
            read_result.is_empty(),
            "Reading from an empty channel should return no queued connections"
        );
    }
}
