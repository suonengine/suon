//! Client leave-npc-shop packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveNpcShopPacket;

impl Decodable for LeaveNpcShopPacket {
    const KIND: PacketKind = PacketKind::LeaveNpcShop;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
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
            LeaveNpcShopPacket::decode(&mut payload).unwrap(),
            LeaveNpcShopPacket
        ));
    }
}
