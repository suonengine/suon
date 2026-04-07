//! Client logout packet.

use super::prelude::*;

/// Packet sent by the client to log out without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::LogoutPacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = LogoutPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, LogoutPacket));
/// ```
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
    fn should_decode_logout_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = LogoutPacket::decode(&mut payload)
            .expect("Logout packets should decode without payload bytes");

        assert!(matches!(packet, LogoutPacket));
        assert!(
            payload.is_empty(),
            "Logout decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_logout_kind_constant() {
        assert_eq!(
            LogoutPacket::KIND,
            PacketKind::Logout,
            "Logout packets should advertise the correct packet kind"
        );
    }
}
