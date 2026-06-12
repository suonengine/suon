use std::sync::Arc;

use serde::Serialize;
use tracing::error;

use crate::connection::{id::ConnectionId, manager::ConnectionManager, stats::ConnectionStats};

/// HTTP response representation for the REST API.
#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
}

/// Minimal REST API for monitoring and managing connections.
///
/// Returns pre-serialized responses that can be served by any HTTP
/// framework (Axum, Actix, Hyper, etc.).
#[allow(dead_code)]
pub struct HttpManager {
    manager: Arc<ConnectionManager>,
}

impl HttpManager {
    pub fn new(manager: Arc<ConnectionManager>) -> Self {
        HttpManager { manager }
    }

    /// GET /api/connections — list all active connections as JSON.
    pub fn list_connections(&self) -> HttpResponse {
        let connections = self.manager.active_connections();
        let body = match serde_json::to_string_pretty(&connections) {
            Ok(json) => json,
            Err(e) => {
                error!(target: "Http", "Failed to serialize connections list: {e}");
                String::new()
            }
        };
        HttpResponse {
            status: 200,
            body,
            content_type: "application/json",
        }
    }

    /// GET /api/connections/{id} — details for a single connection.
    pub fn get_connection(&self, id: ConnectionId) -> HttpResponse {
        match self.manager.get(id) {
            Some(handle) => {
                let info = format!(
                    "{{ \"id\": {}, \"peer\": \"{}\", \"connected\": true }}",
                    id,
                    handle.addr()
                );
                HttpResponse {
                    status: 200,
                    body: info,
                    content_type: "application/json",
                }
            }
            None => HttpResponse {
                status: 404,
                body: format!("{{ \"error\": \"connection {} not found\" }}", id),
                content_type: "application/json",
            },
        }
    }

    /// DELETE /api/connections/{id} — force-close a connection.
    pub fn close_connection(&self, id: ConnectionId) -> HttpResponse {
        match self.manager.get(id) {
            Some(handle) => {
                drop(handle.close());
                HttpResponse {
                    status: 200,
                    body: format!("{{ \"closed\": {} }}", id),
                    content_type: "application/json",
                }
            }
            None => HttpResponse {
                status: 404,
                body: format!("{{ \"error\": \"connection {} not found\" }}", id),
                content_type: "application/json",
            },
        }
    }

    /// GET /api/stats — aggregate connection statistics as JSON.
    pub fn stats(&self) -> HttpResponse {
        let stats: &ConnectionStats = &self.manager.stats;
        let body = format!(
            "{{ \"active\": {}, \"total_accepted\": {}, \"total_closed\": {} }}",
            self.manager.count(),
            stats
                .total_accepted
                .load(std::sync::atomic::Ordering::Relaxed),
            stats
                .total_closed
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        HttpResponse {
            status: 200,
            body,
            content_type: "application/json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HttpManager;
    use crate::{
        connection::{id::ConnectionId, manager::ConnectionManager},
        server::tcp::ProtocolSettings,
    };
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        sync::Arc,
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

    fn setup() -> (Arc<ConnectionManager>, HttpManager) {
        let manager = Arc::new(ConnectionManager::new(0));
        let http = HttpManager::new(manager.clone());
        (manager, http)
    }

    #[test]
    fn http_list_connections_empty() {
        let (_, http) = setup();
        let response = http.list_connections();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("[]") || response.body.contains("\"id\""));
    }

    #[test]
    fn http_get_connection_not_found() {
        let (_, http) = setup();
        let id = ConnectionId::new(0, 999);
        let response = http.get_connection(id);
        assert_eq!(response.status, 404);
        assert!(response.body.contains("not found"));
    }

    #[test]
    fn http_get_connection_found() {
        let (manager, http) = setup();
        let id = manager.register(
            test_peer(),
            test_protocol(),
            crossbeam_channel::bounded(16).0,
        );
        let response = http.get_connection(id);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(&id.to_string()));
    }

    #[test]
    fn http_close_connection_not_found() {
        let (_, http) = setup();
        let id = ConnectionId::new(0, 999);
        let response = http.close_connection(id);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn http_close_connection_found() {
        let (manager, http) = setup();
        let id = manager.register(
            test_peer(),
            test_protocol(),
            crossbeam_channel::bounded(16).0,
        );
        let response = http.close_connection(id);
        assert_eq!(response.status, 200);
    }

    #[test]
    fn http_stats_empty() {
        let (_, http) = setup();
        let response = http.stats();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("\"active\": 0"));
    }

    #[test]
    fn http_stats_after_register() {
        let (manager, http) = setup();
        manager.register(
            test_peer(),
            test_protocol(),
            crossbeam_channel::bounded(16).0,
        );
        let response = http.stats();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("\"active\": 1"));
    }
}
