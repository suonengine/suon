use suon_channel::TaskHandler;
use suon_lua::LuaVm;
use suon_macros::Task;
use suon_resource::Resources;

use crate::{connection::id::ConnectionId, pool::NetworkBufferPool};

#[derive(Task)]
pub struct RawPacket {
    pub id: ConnectionId,
    pub data: Vec<u8>,
}

impl TaskHandler for RawPacket {
    fn run(&mut self, resources: &mut Resources) {
        let lua_vm = resources.get::<LuaVm>();
        if let Err(error) =
            lua_vm.trigger_event("RawPacketEvent", (self.id.as_u64(), self.data.as_slice()))
        {
            tracing::error!(target: "TCP", "RawPacket error: {error}");
        }

        let buffer_pool = &resources.get::<NetworkBufferPool>().0;
        buffer_pool.release(std::mem::take(&mut self.data));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use suon_channel::buffer_pool::BufferPool;

    use super::*;

    #[test]
    fn raw_packet_fields() {
        let packet = RawPacket {
            id: ConnectionId::new(0, 1),
            data: vec![0xAB, 0xCD],
        };
        assert_eq!(packet.id.sequence(), 1);
        assert_eq!(packet.data, vec![0xAB, 0xCD]);
    }

    #[test]
    fn raw_packet_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        let pool = Arc::new(BufferPool::new(4096, 8));
        resources.insert(NetworkBufferPool(pool));
        resources.insert(suon_lua::LuaVm::new());
        resources.insert(suon_channel::Channel::default());
        let mut task = Box::new(RawPacket {
            id: ConnectionId::new(0, 3),
            data: vec![0xAB],
        });
        task.run(&mut resources);
    }

    #[test]
    fn raw_packet_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<RawPacket>();
    }
}
