use suon_channel::TaskHandler;
use suon_lua::LuaVm;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

#[derive(Task)]
pub(crate) struct ConnectionEnd {
    pub id: ConnectionId,
}

impl TaskHandler for ConnectionEnd {
    fn run(&mut self, resources: &mut Resources) {
        let lua_vm = resources.get::<LuaVm>();
        if let Err(error) = lua_vm.trigger_event("ConnectionEndEvent", (self.id.as_u64(),)) {
            tracing::error!(target: "TCP", "ConnectionEnd error: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_end_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        resources.insert(LuaVm::new());
        resources.insert(suon_channel::Channel::default());
        let mut task = Box::new(ConnectionEnd {
            id: ConnectionId::new(0, 1),
        });
        task.run(&mut resources);
    }
}
