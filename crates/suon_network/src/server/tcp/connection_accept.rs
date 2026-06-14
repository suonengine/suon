use std::{net::SocketAddr, sync::Arc, time::Duration};

use tracing::warn;

use tokio::net::TcpStream;

use crate::{
    connection::{id::ConnectionId, manager::ConnectionManager},
    protocol::command::Command,
    server::throttle::ConnectionPermit,
};

/// Result of deciding whether to accept or reject a pending connection.
pub(crate) enum AcceptOutcome {
    Spawn {
        stream: TcpStream,
        command_receiver: crossbeam_channel::Receiver<Command>,
        permit: ConnectionPermit,
    },
    Reject,
}

/// Holds all the state needed to wait for Lua's decision on a new
/// connection and produce an [`AcceptOutcome`].
pub(crate) struct ConnectionAccept {
    pub id: ConnectionId,
    pub address: SocketAddr,
    pub stream: TcpStream,
    pub permit: ConnectionPermit,
    pub manager: Arc<ConnectionManager>,
    pub command_receiver: crossbeam_channel::Receiver<Command>,
    pub begin_response_receiver: tokio::sync::oneshot::Receiver<bool>,
    pub connection_timeout: Duration,
}

impl ConnectionAccept {
    pub async fn decide(self) -> AcceptOutcome {
        let ConnectionAccept {
            id: identifier,
            address: peer,
            stream,
            permit,
            manager,
            command_receiver,
            begin_response_receiver,
            connection_timeout: timeout,
        } = self;

        match tokio::time::timeout(timeout, begin_response_receiver).await {
            Ok(Ok(true)) => AcceptOutcome::Spawn {
                stream,
                command_receiver,
                permit,
            },
            Ok(Ok(false)) => {
                manager.unregister(identifier);
                warn!(target: "TCP", "Lua rejected connection {identifier} from {peer}");
                // stream + permit dropped at end of scope
                AcceptOutcome::Reject
            }
            Ok(Err(_)) => {
                manager.unregister(identifier);
                warn!(target: "TCP", "Lua handler dropped the begin oneshot for {identifier}");
                AcceptOutcome::Reject
            }
            Err(_) => {
                manager.unregister(identifier);
                warn!(
                    target: "TCP",
                    "onConnect timed out for {identifier} from {peer} after {timeout:?}",
                );
                AcceptOutcome::Reject
            }
        }
    }
}
