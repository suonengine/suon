use suon_channel::TaskHandler;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

#[derive(Task)]
pub(crate) struct ConnectionBegin {
    pub id: ConnectionId,
    pub ip: String,
    pub port: u16,
}

impl TaskHandler for ConnectionBegin {
    fn run(self: Box<Self>, _: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_begin_fields() {
        let task = ConnectionBegin {
            id: ConnectionId::new(0, 42),
            ip: "192.168.1.1".into(),
            port: 7171,
        };
        assert_eq!(task.id.sequence(), 42);
        assert_eq!(task.ip, "192.168.1.1");
        assert_eq!(task.port, 7171);
    }

    #[test]
    fn connection_begin_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        let task = Box::new(ConnectionBegin {
            id: ConnectionId::new(0, 1),
            ip: "127.0.0.1".into(),
            port: 7171,
        });
        task.run(&mut resources);
    }
}
