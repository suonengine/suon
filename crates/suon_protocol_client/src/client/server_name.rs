use suon_protocol::prelude::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerNamePacket {
    pub name: String,
}

impl Decodable for ServerNamePacket {
    const KIND: PacketKind = PacketKind::ServerName;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let name = bytes.get_str()?.to_owned();
        Ok(ServerNamePacket { name })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_server_name_from_length_prefixed_payload() {
        let mut payload: &[u8] = &[4, 0, b's', b'u', b'o', b'n'];
        let packet =
            ServerNamePacket::decode(&mut payload).expect("ServerNamePacket should decode");
        assert_eq!(packet.name, "suon");
        assert!(payload.is_empty());
    }
}
