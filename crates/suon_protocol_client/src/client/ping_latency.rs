//! Client ping-latency packet.

use super::prelude::*;

/// Packet sent by the client to request a latency measurement without payload data.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[];
/// let packet = PingLatencyPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, PingLatencyPacket));
/// ```
pub struct PingLatencyPacket;

impl Decodable for PingLatencyPacket {
    const KIND: PacketKind = PacketKind::PingLatency;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(PingLatencyPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_ping_latency_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = PingLatencyPacket::decode(&mut payload)
            .expect("PingLatency packets should decode without payload bytes");

        assert!(matches!(packet, PingLatencyPacket));

        assert!(
            payload.is_empty(),
            "PingLatency decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_ping_latency_kind_constant() {
        assert_eq!(
            PingLatencyPacket::KIND,
            PacketKind::PingLatency,
            "PingLatency packets should advertise the correct packet kind"
        );
    }
}
