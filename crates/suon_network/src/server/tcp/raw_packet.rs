use suon_channel::TaskHandler;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

#[derive(Task)]
pub struct RawPacket {
    pub id: ConnectionId,
    pub data: Vec<u8>,
}

impl TaskHandler for RawPacket {
    fn run(self: Box<Self>, _: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_packet_fields() {
        let p = RawPacket {
            id: ConnectionId::new(0, 1),
            data: vec![0xAB, 0xCD],
        };
        assert_eq!(p.id.sequence(), 1);
        assert_eq!(p.data, vec![0xAB, 0xCD]);
    }

    #[test]
    fn raw_packet_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        let task = Box::new(RawPacket {
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
