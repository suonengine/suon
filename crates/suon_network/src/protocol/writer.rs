use std::io::Write;

use flate2::{Compression, write::DeflateEncoder};
use suon_xtea::ExpandedKey;
use tracing::error;

use crate::server::tcp::protocol::{self, ProtocolSettings, SEQ_FIELD_LEN, SIZE_FIELD_LEN};

/// Bit flag indicating the packet payload is zlib-compressed.
const COMPRESSION_FLAG: u32 = 0x8000_0000;

/// Minimum plaintext size (in bytes) before compression is attempted.
const COMPRESSION_THRESHOLD: usize = 128;

pub struct PacketWriter {
    protocol: ProtocolSettings,
    xtea_key: Option<ExpandedKey>,
    xtea_enabled: bool,
    buffer: Vec<u8>,
    max_buffer_size: usize,
    sequence_id: u32,
}

impl PacketWriter {
    pub fn new(protocol: ProtocolSettings, max_buffer_size: usize) -> Self {
        PacketWriter {
            protocol,
            xtea_key: None,
            xtea_enabled: protocol.uses_xtea,
            buffer: Vec::with_capacity(max_buffer_size),
            max_buffer_size,
            sequence_id: 0,
        }
    }

    pub fn with_xtea_key(mut self, key: [u32; 4]) -> Self {
        self.xtea_key = Some(suon_xtea::expand(&key));
        self
    }

    pub fn with_xtea_enabled(mut self, enabled: bool) -> Self {
        self.xtea_enabled = enabled;
        self
    }

    pub fn with_max_buffer_size(mut self, size: usize) -> Self {
        self.max_buffer_size = size;
        self
    }

    pub fn set_xtea_key(&mut self, key: [u32; 4]) {
        self.xtea_key = Some(suon_xtea::expand(&key));
    }

    pub fn set_xtea_enabled(&mut self, enabled: bool) {
        self.xtea_enabled = enabled;
    }

    pub fn set_max_buffer_size(&mut self, size: usize) {
        self.max_buffer_size = size;
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn should_flush_by_size(&self) -> bool {
        self.buffer.len() >= self.max_buffer_size
    }

    pub fn send(&mut self, plaintext: &[u8]) {
        let framed = self.frame_packet(plaintext);
        self.buffer.extend_from_slice(&framed);
    }

    pub fn send_raw(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
    }

    pub fn take_buffer(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.buffer)
    }

    fn frame_packet(&mut self, plaintext: &[u8]) -> Vec<u8> {
        if self.xtea_enabled && self.protocol.uses_xtea {
            self.frame_xtea_packet(plaintext)
        } else if self.protocol.has_checksum {
            self.frame_checksum_packet(plaintext)
        } else {
            self.frame_plain_packet(plaintext)
        }
    }

    fn frame_plain_packet(&self, plaintext: &[u8]) -> Vec<u8> {
        let size = plaintext.len() as u16;
        let mut out = Vec::with_capacity(SIZE_FIELD_LEN + plaintext.len());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(plaintext);
        out
    }

    fn frame_checksum_packet(&self, plaintext: &[u8]) -> Vec<u8> {
        let checksum = suon_adler32::generate(plaintext);
        let size = (SEQ_FIELD_LEN + plaintext.len()) as u16;
        let mut out = Vec::with_capacity(SIZE_FIELD_LEN + SEQ_FIELD_LEN + plaintext.len());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&checksum.to_le_bytes());
        out.extend_from_slice(plaintext);
        out
    }

    fn frame_xtea_packet(&mut self, plaintext: &[u8]) -> Vec<u8> {
        let seq_field = self.next_sequence_id();
        let Some(key) = self.xtea_key.as_ref() else {
            return self.frame_checksum_packet(plaintext);
        };

        let payload = if plaintext.len() >= COMPRESSION_THRESHOLD {
            let compressed = {
                let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
                if let Err(e) = encoder.write_all(plaintext) {
                    error!(target: "Writer", "Deflate compression error during XTEA packet framing: {e}");
                }

                encoder.finish().ok()
            };

            if let Some(ref compressed) = compressed {
                if compressed.len() < plaintext.len() {
                    let mut padded = protocol::xtea_pad(compressed);
                    suon_xtea::encrypt(&mut padded, key).ok();
                    (padded, seq_field | COMPRESSION_FLAG)
                } else {
                    let mut padded = protocol::xtea_pad(plaintext);
                    suon_xtea::encrypt(&mut padded, key).ok();
                    (padded, seq_field)
                }
            } else {
                let mut padded = protocol::xtea_pad(plaintext);
                suon_xtea::encrypt(&mut padded, key).ok();
                (padded, seq_field)
            }
        } else {
            let mut padded = protocol::xtea_pad(plaintext);
            suon_xtea::encrypt(&mut padded, key).ok();
            (padded, seq_field)
        };

        let total_body = SEQ_FIELD_LEN + payload.0.len();
        let mut out = Vec::with_capacity(SIZE_FIELD_LEN + total_body);
        out.extend_from_slice(&(total_body as u16).to_le_bytes());
        out.extend_from_slice(&payload.1.to_le_bytes());
        out.extend_from_slice(&payload.0);
        out
    }

    fn next_sequence_id(&mut self) -> u32 {
        let seq = self.sequence_id;
        self.sequence_id = self.sequence_id.wrapping_add(1);
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::DeflateDecoder;
    use std::io::Read;
    use suon_xtea::{Key, decrypt, expand};

    fn test_key() -> Key {
        [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210]
    }

    fn decrypt_xtea_framed(framed: &[u8], key: Key) -> Vec<u8> {
        let expanded = &expand(&key);
        let seq_field = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
        let encrypted = &framed[6..];
        let padded_len = encrypted.len().next_multiple_of(8);
        let mut buf = Vec::with_capacity(padded_len);
        buf.extend_from_slice(encrypted);
        buf.resize(padded_len, 0);
        decrypt(&mut buf, expanded).expect("failed to decrypt XTEA data in test helper");

        let padding = buf[0] as usize;
        let data_end = buf.len().saturating_sub(padding);
        let unpadded = if data_end > 1 {
            buf.copy_within(1..data_end, 0);
            buf.truncate(data_end - 1);
            buf
        } else {
            return Vec::new();
        };

        if seq_field & COMPRESSION_FLAG != 0 {
            let mut decoder = DeflateDecoder::new(&unpadded[..]);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .expect("failed to decompress XTEA data in test helper");
            decompressed
        } else {
            unpadded
        }
    }

    #[test]
    fn status_checksum_framing() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"test");

        let framed = writer.take_buffer();
        assert_eq!(framed.len(), 2 + 4 + 4);
        let size = u16::from_le_bytes([framed[0], framed[1]]);
        assert_eq!(size, 8);
        let checksum = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
        assert_eq!(checksum, suon_adler32::generate(b"test"));
        assert_eq!(&framed[6..], b"test");
    }

    #[test]
    fn status_checksum_various_sizes() {
        for len in [1, 2, 3, 7, 15, 100, 1024] {
            let data = vec![0xABu8; len];
            let mut writer = PacketWriter::new(
                ProtocolSettings {
                    header_size: 2,
                    has_checksum: true,
                    uses_xtea: false,
                    uses_rsa: false,
                },
                4096,
            );
            writer.send(&data);
            let framed = writer.take_buffer();
            assert_eq!(framed.len(), 2 + 4 + len);
            let stored = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
            assert_eq!(stored, suon_adler32::generate(&data), "len={len}");
            assert_eq!(&framed[6..], &data, "len={len}");
        }
    }

    #[test]
    fn login_without_xtea_checksum_framing() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: false,
            },
            4096,
        );
        writer.set_xtea_enabled(false);
        writer.send(b"login data");

        let framed = writer.take_buffer();
        assert_eq!(framed.len(), 2 + 4 + 10);
        let checksum = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
        assert_eq!(checksum, suon_adler32::generate(b"login data"));
        assert_eq!(&framed[6..], b"login data");
    }

    #[test]
    fn game_without_xtea_checksum_framing() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_enabled(false);
        writer.send(b"raw");

        let framed = writer.take_buffer();
        assert_eq!(framed.len(), 2 + 4 + 3);
        let checksum = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
        assert_eq!(checksum, suon_adler32::generate(b"raw"));
        assert_eq!(&framed[6..], b"raw");
    }

    #[test]
    fn game_xtea_roundtrip() {
        let key = test_key();
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_key(key);
        writer.send(b"secret data");

        let framed = writer.take_buffer();
        let body_size = u16::from_le_bytes([framed[0], framed[1]]) as usize;
        assert_eq!(body_size + 2, framed.len());

        let unpadded = decrypt_xtea_framed(&framed, key);
        assert_eq!(unpadded, b"secret data");
    }

    #[test]
    fn game_xtea_multiple_sends() {
        let key = test_key();
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_key(key);
        writer.send(b"packet1");
        writer.send(b"packet2");

        let framed = writer.take_buffer();
        let (s1, _) = framed.split_at(2 + u16::from_le_bytes([framed[0], framed[1]]) as usize);

        let p1 = decrypt_xtea_framed(s1, key);
        assert_eq!(p1, b"packet1");
    }

    #[test]
    fn login_xtea_roundtrip() {
        let key = test_key();
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: false,
            },
            4096,
        );
        writer.set_xtea_key(key);
        writer.send(b"login secret");

        let framed = writer.take_buffer();
        let unpadded = decrypt_xtea_framed(&framed, key);
        assert_eq!(unpadded, b"login secret");
    }

    #[test]
    fn xtea_empty_data() {
        let key = test_key();
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_key(key);
        writer.send(b"");

        let framed = writer.take_buffer();
        let unpadded = decrypt_xtea_framed(&framed, key);
        assert!(unpadded.is_empty());
    }

    #[test]
    fn xtea_without_key_falls_back_to_checksum() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.send(b"fallback");
        let framed = writer.take_buffer();

        let size = u16::from_le_bytes([framed[0], framed[1]]) as usize;
        assert_eq!(size + 2, framed.len());
        assert_eq!(&framed[6..], b"fallback");
    }

    #[test]
    fn login_xtea_without_key_falls_back_to_checksum() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"nokey");
        let framed = writer.take_buffer();

        assert_eq!(framed.len(), 2 + 4 + 5);
        let checksum = u32::from_le_bytes([framed[2], framed[3], framed[4], framed[5]]);
        assert_eq!(checksum, suon_adler32::generate(b"nokey"));
    }

    #[test]
    fn multiple_sends_accumulate() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"aaa");
        writer.send(b"bbb");

        let buf = writer.take_buffer();
        let per_packet = 2 + 4 + 3;
        assert_eq!(buf.len(), per_packet * 2);

        let s1 = u16::from_le_bytes([buf[0], buf[1]]);
        assert_eq!(s1, 7);
        assert_eq!(&buf[6..9], b"aaa");

        let s2 = u16::from_le_bytes([buf[9], buf[10]]);
        assert_eq!(s2, 7);
        assert_eq!(&buf[15..18], b"bbb");
    }

    #[test]
    fn sends_accumulate_after_take() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"first");
        writer.take_buffer();

        writer.send(b"second");
        let buf = writer.take_buffer();
        assert_eq!(&buf[6..], b"second");
    }

    #[test]
    fn take_buffer_empty_when_nothing_sent() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        let buf = writer.take_buffer();
        assert!(buf.is_empty());
    }

    #[test]
    fn buffer_size_triggers_flush() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.set_max_buffer_size(12);

        writer.send(b"123");
        assert!(!writer.should_flush_by_size());

        writer.send(b"12345");
        assert!(writer.should_flush_by_size());
    }

    #[test]
    fn buffer_size_exact_boundary() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.set_max_buffer_size(9);
        writer.send(b"123");
        assert!(writer.should_flush_by_size());
    }

    #[test]
    fn buffer_under_size_no_flush() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.set_max_buffer_size(100);
        writer.send(b"small");
        assert!(!writer.should_flush_by_size());
    }

    #[test]
    fn is_empty_after_new() {
        let writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        assert!(writer.is_empty());
    }

    #[test]
    fn is_empty_after_send() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"data");
        assert!(!writer.is_empty());
    }

    #[test]
    fn take_buffer_empties() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        writer.send(b"data");
        writer.take_buffer();
        assert!(writer.is_empty());
        assert_eq!(writer.buffer_len(), 0);
    }

    #[test]
    fn buffer_len_tracks_accumulation() {
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 2,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            },
            4096,
        );
        assert_eq!(writer.buffer_len(), 0);

        writer.send(b"hi");
        assert_eq!(writer.buffer_len(), 2 + 4 + 2);

        writer.send(b"bye");
        assert_eq!(writer.buffer_len(), (2 + 4 + 2) + (2 + 4 + 3));
    }

    #[test]
    fn xtea_compression_roundtrip() {
        let key = test_key();
        let data = vec![0xABu8; 256];
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_key(key);
        writer.send(&data);

        let framed = writer.take_buffer();
        let unpadded = decrypt_xtea_framed(&framed, key);
        assert_eq!(unpadded, data);
    }

    #[test]
    fn xtea_sequence_increments() {
        let key = test_key();
        let mut writer = PacketWriter::new(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            },
            4096,
        );
        writer.set_xtea_key(key);

        writer.send(b"first");
        let f1 = writer.take_buffer();
        let seq1 = u32::from_le_bytes([f1[2], f1[3], f1[4], f1[5]]) & !COMPRESSION_FLAG;

        writer.send(b"second");
        let f2 = writer.take_buffer();
        let seq2 = u32::from_le_bytes([f2[2], f2[3], f2[4], f2[5]]) & !COMPRESSION_FLAG;

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
    }
}
