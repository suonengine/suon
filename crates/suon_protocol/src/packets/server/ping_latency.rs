//! Server ping-latency packet.

use super::prelude::*;

/// Packet sent by the server to respond to latency checks without payload data.
pub struct PingLatencyPacket;

impl Encodable for PingLatencyPacket {
    const KIND: PacketKind = PacketKind::PingLatency;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_ping_latency_with_kind_only() {
        let encoded = PingLatencyPacket.encode_with_kind();

        assert_eq!(
            encoded.as_ref(),
            &[PacketKind::PingLatency as u8],
            "PingLatency packets should encode to just their kind byte"
        );
    }
}
