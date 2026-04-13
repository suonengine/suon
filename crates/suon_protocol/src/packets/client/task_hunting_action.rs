//! Client task-hunting-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to update one task-hunting slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskHuntingAction {
    /// Task slot selected by the client.
    pub slot: u8,

    /// Raw action selector.
    pub action: u8,

    /// Whether the action should upgrade the selected slot.
    pub upgrade: bool,

    /// Race id associated with the task operation.
    pub race_id: u16,
}

impl Decodable for TaskHuntingAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            slot: bytes.get_u8()?,
            action: bytes.get_u8()?,
            upgrade: bytes.get_bool()?,
            race_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_task_hunting_action() {
        let mut payload: &[u8] = &[3, 2, 1, 0x34, 0x12];

        let packet = TaskHuntingAction::decode(PacketKind::TaskHuntingAction, &mut payload)
            .expect("TaskHuntingAction packets should decode slot, action, upgrade, and race id");

        assert_eq!(
            packet,
            TaskHuntingAction {
                slot: 3,
                action: 2,
                upgrade: true,
                race_id: 0x1234,
            }
        );
        assert!(payload.is_empty());
    }
}
