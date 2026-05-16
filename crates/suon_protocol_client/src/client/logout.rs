use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogoutPacket;

impl Decodable for LogoutPacket {
    const KIND: PacketKind = PacketKind::Logout;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(LogoutPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_logout_packet() {
        let mut payload: &[u8] = &[];
        let packet = LogoutPacket::decode(&mut payload).expect("LogoutPacket should decode");
        assert!(matches!(packet, LogoutPacket));
        assert!(payload.is_empty());
    }
}
