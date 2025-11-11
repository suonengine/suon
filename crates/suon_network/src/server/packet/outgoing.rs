use bevy::prelude::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::server::{
    connection::checksum_mode::ChecksumMode,
    packet::{PACKET_CHECKSUM_SIZE, PACKET_HEADER_SIZE},
};

/// Represents a packet that will be sent to a client.
pub(crate) struct OutgoingPacket {
    /// Optional XTEA encryption keys to encrypt the packet payload.
    xtea_key: Option<suon_xtea::XTEAKey>,

    /// Optional checksum mode; determines if and how a checksum is calculated.
    ///
    /// Only applied for protocol versions >= 8.40.
    checksum_mode: ChecksumMode,

    /// Raw bytes of the packet payload before encryption and checksum.
    bytes: Bytes,
}

impl OutgoingPacket {
    pub fn new(bytes: Bytes) -> Self {
        Self {
            xtea_key: None,
            checksum_mode: ChecksumMode::Adler32,
            bytes,
        }
    }

    /// Sets the XTEA encryption key for this packet.
    pub fn xtea_key(&mut self, keys: suon_xtea::XTEAKey) -> &mut Self {
        self.xtea_key = Some(keys);
        self
    }

    /// Sets the checksum mode for this packet.
    pub fn checksum_mode(&mut self, mode: ChecksumMode) -> &mut Self {
        self.checksum_mode = mode;
        self
    }

    /// Encodes the packet into a single preallocated buffer ready for transmission.
    pub fn encode(self) -> Bytes {
        // Extract raw payload data from the internal Bytes buffer
        let payload = self.bytes.chunk();
        let payload_len = payload.len();

        trace!(
            "Encoding outgoing packet ({payload_len} bytes, XTEA enabled={}, checksum mode={:?})",
            self.xtea_key.is_some(),
            self.checksum_mode
        );

        // Preallocate buffer for [header][checksum][payload header][payload].
        let mut buffer = BytesMut::with_capacity(
            PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_HEADER_SIZE + payload_len,
        );

        // Reserve space for header + checksum + payload length
        buffer.put_bytes(
            0,
            PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_HEADER_SIZE,
        );

        // Handle payload encryption if key is present
        match self.xtea_key {
            Some(xtea_key) => {
                trace!("Encrypting payload with XTEA key...");

                let encrypted = suon_xtea::encrypt(payload, &xtea_key);

                // Write encrypted payload length
                buffer[(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE)
                    ..(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_HEADER_SIZE)]
                    .copy_from_slice(&(encrypted.len() as u16).to_le_bytes());

                buffer.extend_from_slice(&encrypted);

                debug!(
                    "XTEA encryption applied: raw={payload_len} bytes → encrypted={} bytes",
                    encrypted.len()
                );
            }
            None => {
                // Write raw payload length
                buffer[(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE)
                    ..(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE + PACKET_HEADER_SIZE)]
                    .copy_from_slice(&(payload_len as u16).to_le_bytes());

                buffer.extend_from_slice(payload);

                debug!("No encryption applied ({payload_len} raw bytes written)");
            }
        }

        // Compute checksum over payload
        let checksum = match self.checksum_mode {
            ChecksumMode::Adler32 => suon_checksum::Adler32Checksum::from(
                &buffer[(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE)..],
            ),
            ChecksumMode::Sequence(..) => {
                unimplemented!();
            }
        };

        // Write checksum
        buffer[PACKET_HEADER_SIZE..(PACKET_HEADER_SIZE + PACKET_CHECKSUM_SIZE)]
            .copy_from_slice(&(*checksum).to_le_bytes());

        debug!(
            "Checksum ({:?}) computed successfully: 0x{:08X} over {} bytes",
            self.checksum_mode,
            *checksum,
            buffer.len() - PACKET_HEADER_SIZE - PACKET_CHECKSUM_SIZE - PACKET_HEADER_SIZE
        );

        // Write total packet length
        let total_len = buffer.len() - PACKET_HEADER_SIZE;
        buffer[..PACKET_HEADER_SIZE].copy_from_slice(&(total_len as u16).to_le_bytes());

        trace!("Final packet size: {total_len} bytes (payload={payload_len})");

        debug!("Packet encoding complete and ready for transmission");

        buffer.freeze()
    }
}
