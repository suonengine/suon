use std::fmt;

use serde::Serialize;

/// A globally unique connection identifier.
///
/// Encodes a server port namespace in the upper 32 bits and a
/// monotonically increasing sequence number in the lower 32 bits,
/// guaranteeing uniqueness across multiple listener ports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ConnectionId(u64);

impl ConnectionId {
    pub const fn new(port_namespace: u32, sequence: u32) -> Self {
        ConnectionId(((port_namespace as u64) << 32) | (sequence as u64))
    }

    pub fn port_namespace(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub fn sequence(&self) -> u32 {
        self.0 as u32
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    /// Reconstruct a [`ConnectionId`] from the raw 64-bit value returned
    /// by [`as_u64`](Self::as_u64).
    pub const fn from_u64(id: u64) -> Self {
        ConnectionId(id)
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ConnectionId> for u64 {
    fn from(id: ConnectionId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::id::*;

    #[test]
    fn connection_id_new_creates_valid_id() {
        let id = ConnectionId::new(1, 42);
        assert_eq!(id.port_namespace(), 1);
        assert_eq!(id.sequence(), 42);
    }

    #[test]
    fn connection_id_zero_namespace() {
        let id = ConnectionId::new(0, 1);
        assert_eq!(id.port_namespace(), 0);
        assert_eq!(id.sequence(), 1);
    }

    #[test]
    fn connection_id_zero_sequence() {
        let id = ConnectionId::new(5, 0);
        assert_eq!(id.port_namespace(), 5);
        assert_eq!(id.sequence(), 0);
    }

    #[test]
    fn connection_id_max_values() {
        let id = ConnectionId::new(u32::MAX, u32::MAX);
        assert_eq!(id.port_namespace(), u32::MAX);
        assert_eq!(id.sequence(), u32::MAX);
    }

    #[test]
    fn connection_id_display() {
        let id = ConnectionId::new(0, 1);
        assert_eq!(id.to_string(), "1");
    }

    #[test]
    fn connection_id_from_u64() {
        let id = ConnectionId::new(0, 42);
        let value: u64 = id.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn connection_id_equality() {
        let a = ConnectionId::new(0, 1);
        let b = ConnectionId::new(0, 1);
        let c = ConnectionId::new(0, 2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn connection_id_namespace_encoding() {
        let id = ConnectionId::new(0xDEAD, 0xBEEF);
        assert_eq!(id.port_namespace(), 0xDEAD);
        assert_eq!(id.sequence(), 0xBEEF);
        assert_eq!(id.as_u64(), ((0xDEADu64) << 32) | 0xBEEFu64);
    }
}
