//! Client ping-latency packet.

use super::prelude::*;

/// Packet sent by the client to request a latency measurement without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::PingLatency};
///
/// let mut payload: &[u8] = &[];
/// let packet = PingLatency::decode(PacketKind::PingLatency, &mut payload).unwrap();
///
/// assert!(matches!(packet, PingLatency));
/// ```
pub struct PingLatency;

impl Decodable for PingLatency {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(PingLatency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_ping_latency_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = PingLatency::decode(PacketKind::PingLatency, &mut payload)
            .expect("PingLatency packets should decode without payload bytes");

        assert!(matches!(packet, PingLatency));

        assert!(
            payload.is_empty(),
            "PingLatency decoding should not consume any payload bytes"
        );
    }
}
