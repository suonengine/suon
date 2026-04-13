//! Client leave-npc-shop packet.

use super::prelude::*;

/// Packet sent by the client to leave the NPC shop flow.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::LeaveNpcShop};
///
/// let mut payload: &[u8] = &[];
/// let packet = LeaveNpcShop::decode(PacketKind::LeaveNpcShop, &mut payload).unwrap();
///
/// assert!(matches!(packet, LeaveNpcShop));
/// assert!(payload.is_empty());
/// ```
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
        let packet = LeaveNpcShop::decode(PacketKind::LeaveNpcShop, &mut payload)
            .expect("LeaveNpcShop packets should decode empty payloads");

        assert!(matches!(packet, LeaveNpcShop));
        assert!(payload.is_empty());
    }
}
