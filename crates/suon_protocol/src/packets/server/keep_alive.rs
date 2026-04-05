//! Server keep-alive packet.

use super::prelude::*;

/// Packet sent by the server to keep the connection active without payload data.
pub struct KeepAlivePacket;

impl Encodable for KeepAlivePacket {
    const KIND: PacketKind = PacketKind::KeepAlive;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_keep_alive_with_kind_only() {
        let encoded = KeepAlivePacket.encode_with_kind();

        assert_eq!(
            encoded.as_ref(),
            &[PacketKind::KeepAlive as u8],
            "KeepAlive packets should encode to just their kind byte"
        );
    }
}
