//! Client open-quest-line packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to open a specific quest line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenQuestLinePacket {
    /// Quest identifier whose line should be fetched.
    pub quest_id: u16,
}

impl Decodable for OpenQuestLinePacket {
    const KIND: PacketKind = PacketKind::OpenQuestLine;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            quest_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_quest_line() {
        let mut payload: &[u8] = &[0x34, 0x12];

        let packet = OpenQuestLinePacket::decode(&mut payload)
            .expect("OpenQuestLine packets should decode the quest id");

        assert_eq!(packet.quest_id, 0x1234);
        assert!(payload.is_empty());
    }
}
