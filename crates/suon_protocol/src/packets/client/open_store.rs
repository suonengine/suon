//! Client open-store packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to open the in-game store.
///
/// Some protocol variants send no payload, while newer clients also include a
/// service type and a category string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenStorePacket {
    /// Optional service-type selector included by newer clients.
    pub service_type: Option<u8>,

    /// Optional category name requested when opening the store.
    pub category: Option<String>,
}

impl Decodable for OpenStorePacket {
    const KIND: PacketKind = PacketKind::OpenStore;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        if bytes.is_empty() {
            return Ok(Self {
                service_type: None,
                category: None,
            });
        }

        Ok(Self {
            service_type: Some(bytes.get_u8()?),
            category: Some(bytes.get_string()?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_empty_open_store_payload() {
        let mut payload: &[u8] = &[];

        let packet = OpenStorePacket::decode(&mut payload)
            .expect("OpenStore packets should accept empty payloads");

        assert_eq!(
            packet,
            OpenStorePacket {
                service_type: None,
                category: None,
            }
        );
    }

    #[test]
    fn should_decode_open_store_with_service_type_and_category() {
        let mut payload: &[u8] = &[2, 4, 0, b'H', b'o', b'm', b'e'];

        let packet = OpenStorePacket::decode(&mut payload)
            .expect("OpenStore packets should decode optional store routing data");

        assert_eq!(packet.service_type, Some(2));
        assert_eq!(packet.category.as_deref(), Some("Home"));
        assert!(payload.is_empty());
    }
}
