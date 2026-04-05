//! Server challenge packet used during authentication.

use bytes::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::packets::encoder::Encoder;

use super::prelude::*;

/// Packet sent by the server to challenge the client during handshake.
///
/// # Examples
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use suon_protocol::packets::server::{Encodable, prelude::ChallengePacket};
///
/// let encoded = ChallengePacket {
///     timestamp: UNIX_EPOCH + Duration::from_secs(42),
///     random_number: 7,
/// }
/// .encode_with_kind();
///
/// assert_eq!(encoded.as_ref(), &[31, 42, 0, 0, 0, 7]);
/// ```
pub struct ChallengePacket {
    /// The moment when the challenge was created.
    pub timestamp: SystemTime,

    /// A single random byte used to add entropy to the handshake.
    pub random_number: u8,
}

impl Encodable for ChallengePacket {
    const KIND: PacketKind = PacketKind::Challenge;

    fn encode(self) -> Option<Bytes> {
        let mut encoder = Encoder::new();
        let timestamp = self
            .timestamp
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0) as u32;

        encoder.put_u32(timestamp);
        encoder.put_u8(self.random_number);

        Some(encoder.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_challenge_timestamp_and_random_number() {
        let packet = ChallengePacket {
            timestamp: UNIX_EPOCH + std::time::Duration::from_secs(42),
            random_number: 7,
        };

        let encoded = packet
            .encode()
            .expect("Challenge packets should always produce a payload");

        assert_eq!(
            encoded.as_ref(),
            &[42, 0, 0, 0, 7],
            "Challenge packets should encode timestamp seconds followed by the random byte"
        );
    }

    #[test]
    fn should_fallback_to_zero_timestamp_when_before_unix_epoch() {
        let packet = ChallengePacket {
            timestamp: UNIX_EPOCH - std::time::Duration::from_secs(1),
            random_number: 99,
        };

        let encoded = packet
            .encode()
            .expect("Challenge packets should always produce a payload");

        assert_eq!(
            encoded.as_ref(),
            &[0, 0, 0, 0, 99],
            "Timestamps before UNIX_EPOCH should serialize as zero seconds"
        );
    }
}
