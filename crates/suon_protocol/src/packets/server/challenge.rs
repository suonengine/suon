use bytes::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::packets::encoder::Encoder;

use super::prelude::*;

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
