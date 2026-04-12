//! Client open-bestiary-overview packet.

use super::prelude::*;

/// Packet sent by the client to open the bestiary overview screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenBestiaryOverviewPacket;

impl Decodable for OpenBestiaryOverviewPacket {
    const KIND: PacketKind = PacketKind::OpenBestiaryOverview;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_bestiary_overview() {
        let mut payload: &[u8] = &[];

        let packet = OpenBestiaryOverviewPacket::decode(&mut payload)
            .expect("OpenBestiaryOverview packets should decode empty payloads");

        assert!(matches!(packet, OpenBestiaryOverviewPacket));
        assert!(payload.is_empty());
    }
}
