use std::net::SocketAddr;
use tracing::trace;

use crossbeam_channel::TrySendError;

use crate::{
    connection::id::ConnectionId,
    protocol::command::{Command, CommandSender},
};

#[derive(Clone)]
pub struct ConnectionHandle {
    id: ConnectionId,
    addr: SocketAddr,
    sender: CommandSender,
}

impl ConnectionHandle {
    pub fn new(id: ConnectionId, addr: SocketAddr, sender: CommandSender) -> Self {
        Self { id, addr, sender }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn send(&self, data: Vec<u8>) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection",
            "Connection {} send {} bytes to {}",
            self.id,
            data.len(),
            self.addr
        );
        self.sender.try_send(Command::Send(data))
    }

    pub fn send_raw(&self, data: Vec<u8>) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection",
            "Connection {} send_raw {} bytes to {}",
            self.id,
            data.len(),
            self.addr
        );
        self.sender.try_send(Command::SendRaw(data))
    }

    pub fn set_xtea_key(&self, key: [u32; 4]) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection", "Connection {} set_xtea_key to {}", self.id, self.addr);
        self.sender.try_send(Command::SetXteaKey(key))
    }

    pub fn set_encryption_enabled(&self, enabled: bool) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection",
            "Connection {} set_encryption_enabled({enabled}) to {}",
            self.id, self.addr
        );
        self.sender.try_send(Command::SetEncryptionEnabled(enabled))
    }

    pub fn set_compression_threshold(&self, threshold: usize) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection",
            "Connection {} set_compression_threshold({threshold}) to {}",
            self.id, self.addr
        );
        self.sender
            .try_send(Command::SetCompressionThreshold(threshold))
    }

    pub fn close_with_reason(&self, reason: String) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection",
            "Connection {} close_with_reason({reason}) to {}",
            self.id, self.addr
        );
        self.sender.try_send(Command::CloseWithReason(reason))
    }

    pub fn close(&self) -> Result<(), TrySendError<Command>> {
        trace!(target: "Connection", "Connection {} close to {}", self.id, self.addr);
        self.sender.try_send(Command::Close)
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionHandle;
    use crate::{connection::id::ConnectionId, protocol::command::Command};
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    fn test_id() -> ConnectionId {
        ConnectionId::new(0, 1)
    }

    fn test_addr() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 100))
    }

    #[test]
    fn handle_send_receives_command_send() {
        let (sender, receiver) = crossbeam_channel::bounded(16);
        let handle = ConnectionHandle::new(test_id(), test_addr(), sender);

        handle
            .send(vec![1, 2, 3])
            .expect("failed to send command in handle_send_receives_command_send");

        let cmd = receiver
            .try_recv()
            .expect("failed to receive command in handle_send_receives_command_send");

        assert!(matches!(cmd, Command::Send(data) if data == vec![1, 2, 3]));
    }

    #[test]
    fn handle_close_receives_command_close() {
        let (sender, receiver) = crossbeam_channel::bounded(16);
        let handle = ConnectionHandle::new(test_id(), test_addr(), sender);

        handle
            .close()
            .expect("failed to close handle in handle_close_receives_command_close");

        let cmd = receiver
            .try_recv()
            .expect("failed to receive Close command in test");

        assert!(matches!(cmd, Command::Close));
    }

    #[test]
    fn handle_send_full_channel_returns_error() {
        let (sender, rx) = crossbeam_channel::bounded(1);
        let handle = ConnectionHandle::new(test_id(), test_addr(), sender);

        handle
            .send(vec![1])
            .expect("failed to send first command in backpressure test");

        let result = handle.send(vec![2]);
        assert!(result.is_err());
        drop(rx);
    }

    #[test]
    fn handle_set_xtea_key() {
        let (sender, receiver) = crossbeam_channel::bounded(16);
        let handle = ConnectionHandle::new(test_id(), test_addr(), sender);

        handle
            .set_xtea_key([1, 2, 3, 4])
            .expect("failed to set XTEA key in test");

        let cmd = receiver
            .try_recv()
            .expect("failed to receive SetXteaKey command in test");

        assert!(matches!(cmd, Command::SetXteaKey(_)));
    }

    #[test]
    fn handle_close_with_reason() {
        let (sender, receiver) = crossbeam_channel::bounded(16);
        let handle = ConnectionHandle::new(test_id(), test_addr(), sender);

        handle
            .close_with_reason("timeout".into())
            .expect("failed to close with reason in test");

        let cmd = receiver
            .try_recv()
            .expect("failed to receive CloseWithReason command in test");

        assert!(matches!(cmd, Command::CloseWithReason(r) if r == "timeout"));
    }
}
