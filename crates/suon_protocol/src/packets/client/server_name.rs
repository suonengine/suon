//! Client server-name packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// First handshake packet sent by the client with the requested server name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerName {
    /// Requested server name decoded from the newline-terminated handshake payload.
    pub server_name: String,
}

impl Decodable for ServerName {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            server_name: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_server_name() {
        let mut payload: &[u8] = &[4, 0, b's', b'u', b'o', b'n'];

        let packet = ServerName::decode(PacketKind::ServerName, &mut payload)
            .expect("ServerName packets should decode the requested server name");

        assert_eq!(packet.server_name, "suon");
        assert!(payload.is_empty());
    }
}
