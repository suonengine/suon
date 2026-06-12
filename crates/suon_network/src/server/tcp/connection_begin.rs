use std::net::SocketAddr;

use suon_channel::TaskHandler;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

#[derive(Task)]
pub(crate) struct ConnectionBegin {
    pub id: ConnectionId,
    pub address: SocketAddr,
}

impl TaskHandler for ConnectionBegin {
    fn run(&mut self, _: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn connection_begin_fields() {
        let task = ConnectionBegin {
            id: ConnectionId::new(0, 42),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 7171),
        };
        assert_eq!(task.id.sequence(), 42);
        assert_eq!(task.address.port(), 7171);
    }

    #[test]
    fn connection_begin_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        let mut task = Box::new(ConnectionBegin {
            id: ConnectionId::new(0, 1),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7171),
        });
        task.run(&mut resources);
    }
}
