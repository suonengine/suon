use suon_channel::TaskHandler;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

#[derive(Task)]
pub(crate) struct ConnectionEnd {
    pub id: ConnectionId,
}

impl TaskHandler for ConnectionEnd {
    fn run(self: Box<Self>, _: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_end_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        let task = Box::new(ConnectionEnd {
            id: ConnectionId::new(0, 1),
        });
        task.run(&mut resources);
    }
}
