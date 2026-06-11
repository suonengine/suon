use std::{fmt, net::SocketAddr, time::Instant};

use serde::Serialize;

use crate::{connection::id::ConnectionId, server::tcp::ProtocolSettings};

/// Runtime information about an active connection.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionInfo {
    pub id: ConnectionId,
    pub peer: SocketAddr,
    pub protocol: ProtocolSettings,
    pub connected_at: u64,
    pub uptime_seconds: u64,
}

impl ConnectionInfo {
    pub fn new(
        id: ConnectionId,
        peer: SocketAddr,
        protocol: ProtocolSettings,
        connected_at: Instant,
    ) -> Self {
        let elapsed = connected_at.elapsed();
        ConnectionInfo {
            id,
            peer,
            protocol,
            connected_at: 0,
            uptime_seconds: elapsed.as_secs(),
        }
    }
}

impl fmt::Display for ConnectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection[id={}, peer={}, protocol={}, uptime={}s]",
            self.id, self.peer, self.protocol, self.uptime_seconds,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionInfo;
    use crate::{connection::id::ConnectionId, server::tcp::ProtocolSettings};
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        time::Instant,
    };

    fn test_peer() -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7000))
    }

    fn test_protocol() -> ProtocolSettings {
        ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        }
    }

    #[test]
    fn connection_info_new_creates_info() {
        let id = ConnectionId::new(0, 1);
        let info = ConnectionInfo::new(id, test_peer(), test_protocol(), Instant::now());
        assert_eq!(info.id, id);
        assert_eq!(info.peer, test_peer());
        assert_eq!(info.protocol, test_protocol());
        assert_eq!(info.uptime_seconds, 0);
    }

    #[test]
    fn connection_info_display() {
        let id = ConnectionId::new(0, 1);
        let info = ConnectionInfo::new(id, test_peer(), test_protocol(), Instant::now());
        let display = info.to_string();
        assert!(display.contains("Connection[id="));
        assert!(display.contains("127.0.0.1"));
        assert!(display.contains("uptime="));
    }

    #[test]
    fn connection_info_uuid_increments() {
        let id1 = ConnectionId::new(0, 1);
        let id2 = ConnectionId::new(0, 2);
        let info1 = ConnectionInfo::new(id1, test_peer(), test_protocol(), Instant::now());
        let info2 = ConnectionInfo::new(id2, test_peer(), test_protocol(), Instant::now());
        assert_ne!(info1.id, info2.id);
    }
}
