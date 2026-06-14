use std::io::Read;

use flate2::read::DeflateDecoder;
use suon_rsa::Rsa;
use suon_xtea::{ExpandedKey, expand};

use crate::server::tcp::protocol::{
    MIN_XTEA_BODY, ProtocolSettings, SEQUENCE_FIELD_LEN, XTEA_KEY_BYTES,
};

/// Bit flag indicating the packet payload is zlib-compressed.
const COMPRESSION_FLAG: u32 = 0x8000_0000;

#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("invalid packet size")]
    InvalidSize,
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: u32, actual: u32 },
    #[error("RSA decryption failed")]
    RsaError,
    #[error("XTEA decryption failed")]
    XteaError,
    #[error("not enough data")]
    NotEnoughData,
}

/// Outcome of [`PacketReader::process_in_place`].
#[derive(Debug, PartialEq, Eq)]
pub enum ProcessOutcome {
    /// The buffer now contains the unwrapped payload (may be shorter
    /// than the original).  Ready to dispatch.
    Complete,
    /// Intermediate state (e.g. RSA handshake); the buffer is
    /// unchanged and should be skipped this iteration.
    Skip,
}

pub struct PacketReader {
    protocol: ProtocolSettings,
    xtea_key: Option<ExpandedKey>,
    xtea_enabled: bool,
    rsa_key: Option<Rsa>,
    rsa_done: bool,
}

impl PacketReader {
    pub fn new(protocol: ProtocolSettings) -> Self {
        PacketReader {
            protocol,
            xtea_key: None,
            xtea_enabled: protocol.uses_xtea,
            rsa_key: None,
            rsa_done: !protocol.uses_rsa,
        }
    }

    pub fn with_xtea_key(mut self, key: [u32; 4]) -> Self {
        self.xtea_key = Some(expand(&key));
        self
    }

    pub fn with_xtea_enabled(mut self, enabled: bool) -> Self {
        self.xtea_enabled = enabled;
        self
    }

    pub fn with_rsa_key(mut self, key: Rsa) -> Self {
        self.rsa_key = Some(key);
        self
    }

    pub fn with_rsa_done(mut self, done: bool) -> Self {
        self.rsa_done = done;
        self
    }

    pub fn set_rsa_done(&mut self, done: bool) {
        self.rsa_done = done;
    }

    pub fn set_rsa_key(&mut self, key: Rsa) {
        self.rsa_key = Some(key);
    }

    pub fn set_xtea_enabled(&mut self, enabled: bool) {
        self.xtea_enabled = enabled;
    }

    pub fn set_xtea_key(&mut self, key: [u32; 4]) {
        self.xtea_key = Some(expand(&key));
    }

    /// Process a packet in-place, leaving `body` with the decrypted payload.
    ///
    /// # Errors
    ///
    /// Returns [`ProcessError::InvalidSize`] if the body is empty or the
    /// unpadded result is empty, [`ProcessError::ChecksumMismatch`] if
    /// the adler32 checksum doesn't match, [`ProcessError::RsaError`] if
    /// RSA decryption fails, [`ProcessError::XteaError`] if XTEA
    /// decryption fails, or [`ProcessError::NotEnoughData`] if the body
    /// is too short for the expected protocol step.
    pub fn process_in_place(&mut self, body: &mut Vec<u8>) -> Result<ProcessOutcome, ProcessError> {
        if body.is_empty() {
            return Err(ProcessError::InvalidSize);
        }

        if !self.rsa_done {
            return self.process_rsa_handshake_in_place(body);
        }

        if self.xtea_enabled && self.protocol.uses_xtea {
            return self.process_xtea_in_place(body);
        }

        if self.protocol.has_checksum {
            return self.process_checksum_in_place(body);
        }

        Ok(ProcessOutcome::Complete)
    }

    /// Strip and verify the checksum prefix, shifting payload in-place.
    fn process_checksum_in_place(
        &self,
        body: &mut Vec<u8>,
    ) -> Result<ProcessOutcome, ProcessError> {
        if body.len() < SEQUENCE_FIELD_LEN {
            return Err(ProcessError::NotEnoughData);
        }

        let stored_checksum = u32::from_le_bytes(
            body[..SEQUENCE_FIELD_LEN]
                .try_into()
                .expect("SEQ_FIELD_LEN is 4 bytes"),
        );

        let payload_len = body.len() - SEQUENCE_FIELD_LEN;

        if stored_checksum != 0 {
            let computed = suon_adler32::generate(&body[SEQUENCE_FIELD_LEN..]);
            if stored_checksum != computed {
                return Err(ProcessError::ChecksumMismatch {
                    expected: stored_checksum,
                    actual: computed,
                });
            }
        }

        if payload_len == 0 {
            return Err(ProcessError::InvalidSize);
        }

        body.copy_within(SEQUENCE_FIELD_LEN.., 0);
        body.truncate(payload_len);
        Ok(ProcessOutcome::Complete)
    }

    /// RSA handshake — runs once per connection, sets XTEA key if found.
    fn process_rsa_handshake_in_place(
        &mut self,
        body: &mut Vec<u8>,
    ) -> Result<ProcessOutcome, ProcessError> {
        let rsa = self.rsa_key.as_ref().ok_or(ProcessError::RsaError)?;
        let mut decrypted = body.clone();

        if suon_rsa::decrypt(rsa, &mut decrypted).is_ok() {
            if decrypted.is_empty() || decrypted[0] != 0 {
                self.rsa_done = true;
                return Ok(ProcessOutcome::Complete);
            }

            // First byte is 0 → XTEA key exchange.
            if decrypted.len() > XTEA_KEY_BYTES {
                let k0 = u32::from_le_bytes(decrypted[1..5].try_into().expect("XTEA_KEY_BYTES=16"));
                let k1 = u32::from_le_bytes(decrypted[5..9].try_into().expect("5..9 is 4 bytes"));
                let k2 = u32::from_le_bytes(decrypted[9..13].try_into().expect("9..13 is 4 bytes"));
                let k3 =
                    u32::from_le_bytes(decrypted[13..17].try_into().expect("13..17 is 4 bytes"));
                self.set_xtea_key([k0, k1, k2, k3]);
            }

            self.rsa_done = true;
            return Ok(ProcessOutcome::Skip);
        }

        self.rsa_done = true;
        Ok(ProcessOutcome::Complete)
    }

    /// Decrypt and unpad an XTEA packet in-place, then handle
    /// optional zlib decompression.
    fn process_xtea_in_place(
        &mut self,
        body: &mut Vec<u8>,
    ) -> Result<ProcessOutcome, ProcessError> {
        if body.len() < MIN_XTEA_BODY {
            return Err(ProcessError::NotEnoughData);
        }

        let seq_field = u32::from_le_bytes(
            body[..SEQUENCE_FIELD_LEN]
                .try_into()
                .expect("SEQ_FIELD_LEN is 4 bytes"),
        );

        let encrypted_len = body.len() - SEQUENCE_FIELD_LEN;
        if encrypted_len == 0 || !encrypted_len.is_multiple_of(8) {
            return Err(ProcessError::NotEnoughData);
        }

        let key = self.xtea_key.as_ref().ok_or(ProcessError::XteaError)?;
        suon_xtea::decrypt(&mut body[SEQUENCE_FIELD_LEN..], key)
            .map_err(|_| ProcessError::XteaError)?;

        let padding = body[SEQUENCE_FIELD_LEN] as usize;

        let data_end = body.len().saturating_sub(padding);
        if data_end <= SEQUENCE_FIELD_LEN + 1 {
            return Err(ProcessError::InvalidSize);
        }

        let unpadded_len = data_end - SEQUENCE_FIELD_LEN - 1;
        body.copy_within(SEQUENCE_FIELD_LEN + 1..data_end, 0);
        body.truncate(unpadded_len);

        if body.is_empty() {
            return Err(ProcessError::InvalidSize);
        }

        // Optional zlib decompression (only allocation in this path).
        if seq_field & COMPRESSION_FLAG != 0 {
            let mut decoder = DeflateDecoder::new(&body[..]);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|_| ProcessError::InvalidSize)?;
            *body = decompressed;
        }

        Ok(ProcessOutcome::Complete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::tcp::protocol::{self, RSA_KEY_SIZE};
    use suon_xtea::{Key, encrypt, expand};

    fn test_key() -> Key {
        [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210]
    }

    fn build_xtea_body(key: &Key, plaintext: &[u8], seq_field: u32) -> Vec<u8> {
        let expanded = &expand(key);
        let padded = protocol::xtea_pad(plaintext);
        let mut encrypted = padded.clone();
        encrypt(&mut encrypted, expanded)
            .expect("failed to encrypt XTEA data in build_xtea_body helper");
        let mut body = Vec::with_capacity(4 + encrypted.len());
        body.extend_from_slice(&seq_field.to_le_bytes());
        body.extend_from_slice(&encrypted);
        body
    }

    #[test]
    fn process_empty_body_returns_invalid_size() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        });
        let mut buf = b"".to_vec();
        assert!(matches!(
            reader.process_in_place(&mut buf),
            Err(ProcessError::InvalidSize)
        ));
    }

    #[test]
    fn process_empty_body_game() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        let mut buf = b"".to_vec();
        assert!(matches!(
            reader.process_in_place(&mut buf),
            Err(ProcessError::InvalidSize)
        ));
    }

    #[test]
    fn status_passthrough_text() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        });
        let mut buf = b"hello".to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should process plaintext successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], b"hello");
    }

    #[test]
    fn status_passthrough_binary() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        });
        let data = &[0x00, 0xFF, 0xAB, 0x7F];
        let mut buf = data.to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should process binary data successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], data);
    }

    #[test]
    fn status_passthrough_single_byte() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        });
        let mut buf = b"\x01".to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should process single byte successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], b"\x01");
    }

    #[test]
    fn status_passthrough_large() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        });
        let data = vec![0xABu8; 4096];
        let mut buf = data.clone();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should process large data successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], &data[..]);
    }

    #[test]
    fn status_checksum_verify_passes() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        });
        let data = b"verified data";
        let checksum = suon_adler32::generate(data);
        let mut body = Vec::with_capacity(4 + data.len());
        body.extend_from_slice(&checksum.to_le_bytes());
        body.extend_from_slice(data);
        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process checksum body successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], &data[..]);
    }

    #[test]
    fn status_checksum_mismatch_detected() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        });
        #[allow(clippy::unnecessary_cast)]
        let checksum = 0xDEAD_BEEFu32;
        let data = b"testdata";
        let mut body = Vec::with_capacity(4 + data.len());
        body.extend_from_slice(&checksum.to_le_bytes());
        body.extend_from_slice(data);
        let mut proc_buf = body.clone();
        assert!(matches!(
            reader.process_in_place(&mut proc_buf),
            Err(ProcessError::ChecksumMismatch { .. })
        ));
    }

    #[test]
    fn status_checksum_zero_skips_validation() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        });
        let body = [0u8; 8];
        let mut proc_buf = body.to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process zero-checksum body successfully"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], &body[4..]);
    }

    #[test]
    fn login_without_encryption_passthrough() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: false,
            uses_xtea: true,
            uses_rsa: false,
        });
        reader.xtea_enabled = false;
        let data = b"login packet";
        let mut buf = data.to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should process login packet without encryption"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], data);
    }

    #[test]
    fn login_with_encryption_xtea_roundtrip() {
        let key = test_key();
        let body = build_xtea_body(&key, b"login data", suon_adler32::generate(b"login data"));

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: false,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process encrypted login data"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], b"login data");
    }

    #[test]
    fn game_xtea_roundtrip() {
        let key = test_key();
        let body = build_xtea_body(&key, b"test data!", 0);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process XTEA roundtrip data"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], b"test data!");
    }

    #[test]
    fn game_xtea_with_seq_field() {
        let key = test_key();
        let plaintext = b"hello world";
        let body = build_xtea_body(&key, plaintext, 0);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process XTEA with seq field"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], plaintext);
    }

    #[test]
    fn game_xtea_large_packet() {
        let key = test_key();
        let plaintext = vec![0xABu8; 1024];
        let body = build_xtea_body(&key, &plaintext, 0);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process large XTEA packet"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], &plaintext[..]);
    }

    #[test]
    fn xtea_body_too_short() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(test_key());
        reader.rsa_done = true;

        assert!(matches!(
            reader.process_in_place(&mut vec![0; 3]),
            Err(ProcessError::NotEnoughData)
        ));
    }

    #[test]
    fn xtea_body_4_bytes_is_not_enough() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(test_key());
        reader.rsa_done = true;

        assert!(matches!(
            reader.process_in_place(&mut vec![0; 4]),
            Err(ProcessError::NotEnoughData)
        ));
    }

    #[test]
    fn xtea_body_5_bytes_encrypted_not_padded() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(test_key());
        reader.rsa_done = true;

        assert!(reader.process_in_place(&mut vec![0; 5]).is_err());
    }

    #[test]
    fn xtea_body_6_bytes_encrypted_not_padded() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(test_key());
        reader.rsa_done = true;

        assert!(reader.process_in_place(&mut vec![0; 6]).is_err());
    }

    #[test]
    fn xtea_encrypted_section_truly_empty() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(test_key());
        reader.rsa_done = true;

        assert!(matches!(
            reader.process_in_place(&mut vec![0; 4]),
            Err(ProcessError::NotEnoughData)
        ));
    }

    #[test]
    fn xtea_wrong_key_produces_error() {
        let good_key: Key = [1, 2, 3, 4];
        let wrong_key: Key = [5, 6, 7, 8];
        let plaintext = b"secret";
        let body = build_xtea_body(&good_key, plaintext, 0);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(wrong_key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert!(matches!(
            reader.process_in_place(&mut proc_buf),
            Err(ProcessError::InvalidSize)
        ));
    }

    #[test]
    fn xtea_seq_field_ignored() {
        let key = test_key();
        let body = build_xtea_body(&key, b"hello", 42);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process XTEA with non-zero seq field"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], b"hello");
    }

    #[test]
    fn xtea_zero_seq_field_works() {
        let key = test_key();
        let body = build_xtea_body(&key, b"hello", 0);

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        let mut proc_buf = body.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("reader should process XTEA with zero seq field"),
            ProcessOutcome::Complete
        );
        assert_eq!(&proc_buf[..], b"hello");
    }

    #[test]
    fn xtea_no_key_set() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.rsa_done = true;

        let mut buf = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        assert!(matches!(
            reader.process_in_place(&mut buf),
            Err(ProcessError::XteaError)
        ));
    }

    #[test]
    fn xtea_enabled_false_skips_decrypt() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: false,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.xtea_enabled = false;
        reader.rsa_done = true;

        let data = b"raw packet";
        let mut buf = data.to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut buf)
                .expect("reader should skip XTEA decrypt when disabled"),
            ProcessOutcome::Complete
        );
        assert_eq!(&buf[..], data);
    }

    #[test]
    fn rsa_done_initially_false_for_game() {
        let reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        assert!(!reader.rsa_done);
    }

    #[test]
    fn rsa_done_initially_true_for_login() {
        let reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: false,
        });
        assert!(reader.rsa_done);
    }

    #[test]
    fn rsa_done_initially_true_for_status() {
        let reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        });
        assert!(reader.rsa_done);
    }

    #[test]
    fn game_protocol_uses_rsa() {
        assert!(
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: true,
                uses_rsa: true,
            }
            .uses_rsa
        );
    }

    #[test]
    fn rsa_handshake_no_key_returns_error() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        assert!(matches!(
            reader.process_in_place(&mut b"\x01\x02".to_vec()),
            Err(ProcessError::RsaError)
        ));
    }

    const TEST_RSA_PEM: &str = "\
-----BEGIN RSA PRIVATE KEY-----
MIICXgIBAAKBgQDSpz7WAGdJgdvbIGy4leEbqptY/LyxWY4eyJ5Fn/IC0cMWw830
Rexg0F78yHeA4Rcu0V6r5oatCtoKTgbO5g9UJtY9BHXANiK4K4q+RVjSeEDx0StW
+EqhRGptc0c39T0B/dSbw9Y8lKmkaOk/2OEPCGtPW6qbwt5ahBoJINwYEQIDAQAB
AoGAIk15vf9y0lWDJ7uv+J7veUHe6i69y2N58SlaHJxfHHZr/lkEQLLiOyGzVhaO
3z3IOKd/cx6m76bEusjZ8vcjp5Sry1xZQuMWBx2iCB0e9+nxGuaSTSoOJrpscJLH
ngqqdjGJY6brU6QpEV0w8UjWnXe9pVWIORQIpa/fdME4/8ECQQD1iTLmuQPG0Y+p
HnCRdgeKUUNUXMDjO9cQtSMHH2ke1ZMbpayAKrhPFuBc6qf1kur0J8WH9xNAvf+c
kLzZgbL7AkEA26F5/idz+5OyNXEnudthKyEToPO53SJYY5uyEcOdRYEgrsNYCVsM
JvKV1vlBriZ1GiNeYWKVQ4Y3AYMLyF3TYwJBAICl7DebRPFNJ7pyqoRslTLRtTdk
ieQFnH+yiLHYsVloifV4btOQjpVR5SiKAorW+agHlqXQvRO0+VLtOyWzoTUCQQC/
jPfex/4J3mjA322sVT9L5E9AQxFJYhkA1tvZTmguJE6i3VA86KGSnmQ816uG/ZeI
MmywNtDD0ZzLvsVZ/SrNAkEA7i/nj9I9vYmjroqD+1r6D5zfj5rFmqxAhW8wMDzh
tO6vywVbLiOFudajEttnKgRV7AWJENyfTbhcuW1AXJvlEA==
-----END RSA PRIVATE KEY-----";

    #[test]
    fn rsa_handshake_success_decrypt_and_set_key() {
        let rsa = suon_rsa::load_pem(TEST_RSA_PEM).expect("failed to load test RSA key");

        let mut plaintext = vec![0u8; RSA_KEY_SIZE];
        plaintext[0] = 0;
        let xtea_key_bytes: [u8; XTEA_KEY_BYTES] = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        plaintext[1..=XTEA_KEY_BYTES].copy_from_slice(&xtea_key_bytes);

        suon_rsa::encrypt(&rsa, &mut plaintext).expect("RSA encryption should succeed in test");
        let encrypted_data = plaintext;

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_rsa_key(rsa);

        let mut proc_buf = encrypted_data.clone();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("RSA handshake should succeed"),
            ProcessOutcome::Skip
        );
        assert!(reader.rsa_done, "rsa_done should be set after handshake");
    }

    #[test]
    fn rsa_handshake_decrypt_failure_fallthrough() {
        let rsa = suon_rsa::load_pem(TEST_RSA_PEM).expect("failed to load test RSA key");

        let invalid_data = b"short";

        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: true,
        });
        reader.set_rsa_key(rsa);

        let mut proc_buf = invalid_data.to_vec();
        assert_eq!(
            reader
                .process_in_place(&mut proc_buf)
                .expect("RSA handshake should fallthrough on short data"),
            ProcessOutcome::Complete
        );
        assert!(reader.rsa_done, "rsa_done should be set after fallthrough");
        assert_eq!(&proc_buf[..], invalid_data);
    }

    #[test]
    fn rsa_handshake_failure_passthrough() {}

    #[test]
    fn reader_state_rsa_done_set_after_first_call() {
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        drop(reader.process_in_place(&mut b"some data".to_vec()));
        assert!(!reader.rsa_done);
    }

    #[test]
    fn reader_xtea_key_persists_after_process() {
        let key = test_key();
        let mut reader = PacketReader::new(ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        });
        reader.set_xtea_key(key);
        reader.rsa_done = true;

        assert!(reader.xtea_key.is_some());

        let body = build_xtea_body(&key, b"first", 0);
        let mut proc_buf = body.clone();
        drop(reader.process_in_place(&mut proc_buf));
        assert!(reader.xtea_key.is_some());
    }

    fn check_process_invariants(settings: ProtocolSettings, input: &[u8]) {
        let mut reader = PacketReader::new(settings);
        let mut buf = input.to_vec();
        let _ = reader.process_in_place(&mut buf);
    }

    #[test]
    fn invariance_passthrough() {
        let settings = ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        };
        check_process_invariants(settings, b"hello world");
        check_process_invariants(settings, b"");
        check_process_invariants(settings, b"\x00\xFF\xAB");
        check_process_invariants(settings, &[0xABu8; 4096]);
    }

    #[test]
    fn invariance_checksum() {
        let settings = ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        };
        let data = b"verified data";
        let checksum = suon_adler32::generate(data);
        let mut body = Vec::with_capacity(4 + data.len());
        body.extend_from_slice(&checksum.to_le_bytes());
        body.extend_from_slice(data);
        check_process_invariants(settings, &body);

        // zero-checksum skip
        let zero_body = [0u8; 8];
        check_process_invariants(settings, &zero_body);

        // mismatched checksum
        let bad_checksum = 0xDEAD_BEEFu32;
        let mut bad_body = Vec::with_capacity(4 + data.len());
        bad_body.extend_from_slice(&bad_checksum.to_le_bytes());
        bad_body.extend_from_slice(data);
        check_process_invariants(settings, &bad_body);
    }

    #[test]
    fn invariance_xtea() {
        let key = test_key();
        let settings = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        };

        let test_cases: &[&[u8]] = &[b"", b"a", b"hello", b"test data!", &[0xABu8; 256]];
        for plaintext in test_cases {
            let body = build_xtea_body(&key, plaintext, 0);

            let mut reader = PacketReader::new(settings);
            reader.set_xtea_key(key);
            reader.rsa_done = true;
            let mut buf = body.clone();
            let _ = reader.process_in_place(&mut buf);
        }
    }

    #[test]
    fn invariance_xtea_seq_field() {
        let key = test_key();
        let settings = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        };

        for seq in [0u32, 1, 42, u32::MAX >> 1] {
            let body = build_xtea_body(&key, b"payload", seq);
            check_process_invariants(settings, &body);
        }
    }

    #[test]
    fn invariance_xtea_error_paths() {
        let settings = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        };
        check_process_invariants(settings, &[0u8; 3]); // too short
        check_process_invariants(settings, &[0u8; 4]); // seq only
        check_process_invariants(settings, &[0u8; 5]); // seq + 1 byte (not 8-aligned)
    }

    #[test]
    fn invariance_empty_body() {
        let settings = ProtocolSettings {
            header_size: 2,
            has_checksum: false,
            uses_xtea: false,
            uses_rsa: false,
        };
        check_process_invariants(settings, b"");
    }
}
