//! Client leave-npc-shop packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveNpcShop;

impl Decodable for LeaveNpcShop {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_npc_shop() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            LeaveNpcShop::decode(PacketKind::LeaveNpcShop, &mut payload).unwrap(),
            LeaveNpcShop
        ));
    }
}
