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

    #[test]
    fn should_queue_and_drain_incoming_connections() {
        smol::block_on(async {
            let listener = smol::net::TcpListener::bind(("127.0.0.1", 0))
                .await
                .expect("The test listener should bind successfully");
            let address = listener
                .local_addr()
                .expect("The test listener should expose a local address");

            let accept_task = smol::spawn(async move {
                let (stream, _) = listener
                    .accept()
                    .await
                    .expect("The test listener should accept one client");
                stream
            });

            let client = smol::net::TcpStream::connect(address)
                .await
                .expect("The test client should connect successfully");
            let server_stream = accept_task.await;

            let connections = IncomingConnections::default();
            connections
                .send(server_stream)
                .expect("The queue should accept incoming connections");

            let queued = connections.read();

            assert_eq!(
                queued.len(),
                1,
                "read should drain the queued incoming connection"
            );
            assert!(
                connections.read().is_empty(),
                "Reading again should return an empty list after the queue is drained"
            );

            drop(client);
        });
    }
}
