use std::fmt;

use serde::{Deserialize, Serialize};

/// Number of bytes in the packet size header (u16 length).
pub const SIZE_FIELD_LEN: usize = 2;

/// Number of bytes in the crypto sequence / flags field.
pub const SEQUENCE_FIELD_LEN: usize = 4;

/// Minimum number of bytes in an XTEA frame body (= seq header + 1 encrypted byte).
pub const MIN_XTEA_BODY: usize = SEQUENCE_FIELD_LEN + 1;

/// Number of bytes of XTEA key material extracted from an RSA handshake.
pub const XTEA_KEY_BYTES: usize = 16;

/// 128 bytes = 1024-bit RSA key.
pub const RSA_KEY_SIZE: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ProtocolSettings {
    pub header_size: usize,
    pub has_checksum: bool,
    pub uses_xtea: bool,
    pub uses_rsa: bool,
}

impl Default for ProtocolSettings {
    fn default() -> Self {
        ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        }
    }
}

impl fmt::Display for ProtocolSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hdr={} chk={} xtea={} rsa={}",
            self.header_size, self.has_checksum, self.uses_xtea, self.uses_rsa,
        )
    }
}

pub fn xtea_padding_byte() -> u8 {
    0x33
}

pub fn xtea_pad(data: &[u8]) -> Vec<u8> {
    let padding = (8u8.wrapping_sub(((data.len() as u8) + 1) % 8)) % 8;
    let padded_len = 1 + data.len() + padding as usize;
    let mut out = Vec::with_capacity(padded_len);
    out.push(padding);
    out.extend_from_slice(data);
    out.resize(padded_len, xtea_padding_byte());
    out
}

pub fn xtea_unpad(data: &[u8]) -> &[u8] {
    if data.is_empty() {
        return data;
    }

    let padding = data[0] as usize;
    let end = data.len().saturating_sub(padding);
    let start = 1;
    if start >= end {
        return &[];
    }

    &data[start..end]
}

#[allow(dead_code)]
pub fn read_u16_le(data: &[u8]) -> Option<(u16, &[u8])> {
    if data.len() < 2 {
        return None;
    }

    let value = u16::from_le_bytes([data[0], data[1]]);
    Some((value, &data[2..]))
}

#[allow(dead_code)]
pub fn read_u32_le(data: &[u8]) -> Option<(u32, &[u8])> {
    if data.len() < 4 {
        return None;
    }

    let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    Some((value, &data[4..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xtea_pad_empty() {
        let padded = xtea_pad(b"");
        assert!(padded.len().is_multiple_of(8));
        assert_eq!(padded[0], 7);
        assert_eq!(padded.len(), 8);
        for &b in &padded[1..] {
            assert_eq!(b, xtea_padding_byte());
        }
        assert_eq!(xtea_unpad(&padded), b"");
    }

    #[test]
    fn xtea_pad_single_byte() {
        let padded = xtea_pad(b"a");
        assert_eq!(padded.len(), 8);
        assert_eq!(padded[0], 6);
        assert_eq!(padded[1], b'a');
        let unpadded = xtea_unpad(&padded);
        assert_eq!(unpadded, b"a");
    }

    #[test]
    fn xtea_pad_7_bytes_max_padding() {
        let padded = xtea_pad(b"1234567");
        assert_eq!(padded.len(), 8);
        assert_eq!(padded[0], 0);
        assert_eq!(&padded[1..], b"1234567");
        assert_eq!(xtea_unpad(&padded), b"1234567");
    }

    #[test]
    fn xtea_pad_8_bytes_exact_block() {
        let padded = xtea_pad(b"12345678");
        assert_eq!(padded.len(), 16);
        assert_eq!(padded[0], 7);
        let unpadded = xtea_unpad(&padded);
        assert_eq!(unpadded, b"12345678");
    }

    #[test]
    fn xtea_pad_9_bytes() {
        let padded = xtea_pad(b"123456789");
        assert_eq!(padded.len(), 16);
        assert_eq!(padded[0], 6);
        let unpadded = xtea_unpad(&padded);
        assert_eq!(unpadded, b"123456789");
    }

    #[test]
    fn xtea_pad_deterministic() {
        let a = xtea_pad(b"hello");
        let b = xtea_pad(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn xtea_pad_all_padding_values() {
        for len in 0..16 {
            let data = vec![b'x'; len];
            let padded = xtea_pad(&data);
            assert!(padded.len().is_multiple_of(8));
            let expected_padding = (8u8.wrapping_sub(((len as u8) + 1) % 8)) % 8;
            assert_eq!(padded[0], expected_padding, "len={len}");
            assert_eq!(xtea_unpad(&padded), data.as_slice());
        }
    }

    #[test]
    fn xtea_unpad_empty_data() {
        assert_eq!(xtea_unpad(b""), b"");
    }

    #[test]
    fn xtea_unpad_no_padding_byte() {
        let result = xtea_unpad(b"\x00hello");
        assert_eq!(result, b"hello");
    }

    #[test]
    fn xtea_unpad_zero_length_result() {
        let mut buf = vec![5u8; 6];
        buf[0] = 5;
        let result = xtea_unpad(&buf);
        assert_eq!(result, b"");
    }

    #[test]
    fn xtea_unpad_padding_larger_than_data() {
        let buf = b"\x10hello";
        let result = xtea_unpad(buf);
        assert_eq!(result, b"");
    }

    #[test]
    fn read_u16_le_valid_with_rest() {
        let data = [0x10, 0x00, 0xFF];
        let (value, rest) = read_u16_le(&data).expect("read_u16_le should succeed with 3 bytes");
        assert_eq!(value, 16);
        assert_eq!(rest, &[0xFF]);
    }

    #[test]
    fn read_u16_le_exact_two_bytes() {
        let data = [0xEF, 0xBE];
        let (value, rest) =
            read_u16_le(&data).expect("read_u16_le should succeed with exactly 2 bytes");
        assert_eq!(value, 0xBEEF);
        assert!(rest.is_empty());
    }

    #[test]
    fn read_u16_le_too_short() {
        assert!(read_u16_le(&[0x01]).is_none());
    }

    #[test]
    fn read_u16_le_empty() {
        assert!(read_u16_le(&[]).is_none());
    }

    #[test]
    fn read_u16_le_max_value() {
        let data = [0xFF, 0xFF];
        let (value, _) = read_u16_le(&data).expect("read_u16_le should succeed with max value");
        assert_eq!(value, u16::MAX);
    }

    #[test]
    fn read_u32_le_valid_with_rest() {
        let data = [0x78, 0x56, 0x34, 0x12, 0xAA];
        let (value, rest) = read_u32_le(&data).expect("read_u32_le should succeed with 5 bytes");
        assert_eq!(value, 0x12345678);
        assert_eq!(rest, &[0xAA]);
    }

    #[test]
    fn read_u32_le_exact_four_bytes() {
        let data = [0x01, 0x00, 0x00, 0x00];
        let (value, rest) =
            read_u32_le(&data).expect("read_u32_le should succeed with exactly 4 bytes");
        assert_eq!(value, 1);
        assert!(rest.is_empty());
    }

    #[test]
    fn read_u32_le_too_short() {
        assert!(read_u32_le(&[0; 3]).is_none());
    }

    #[test]
    fn read_u32_le_empty() {
        assert!(read_u32_le(&[]).is_none());
    }

    #[test]
    fn read_u32_le_max_value() {
        let data = [0xFF; 4];
        let (value, _) = read_u32_le(&data).expect("read_u32_le should succeed with max value");
        assert_eq!(value, u32::MAX);
    }

    #[test]
    fn protocol_settings_game() {
        let cfg = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: true,
        };
        assert_eq!(cfg.header_size, 6);
        assert!(cfg.has_checksum);
        assert!(cfg.uses_xtea);
        assert!(cfg.uses_rsa);
    }

    #[test]
    fn protocol_settings_login() {
        let cfg = ProtocolSettings {
            header_size: 6,
            has_checksum: true,
            uses_xtea: true,
            uses_rsa: false,
        };
        assert_eq!(cfg.header_size, 6);
        assert!(cfg.has_checksum);
        assert!(cfg.uses_xtea);
        assert!(!cfg.uses_rsa);
    }

    #[test]
    fn protocol_settings_status() {
        let cfg = ProtocolSettings {
            header_size: 2,
            has_checksum: true,
            uses_xtea: false,
            uses_rsa: false,
        };
        assert_eq!(cfg.header_size, 2);
        assert!(cfg.has_checksum);
        assert!(!cfg.uses_xtea);
        assert!(!cfg.uses_rsa);
    }

    #[test]
    fn protocol_settings_default_is_game() {
        assert_eq!(
            ProtocolSettings::default(),
            ProtocolSettings {
                header_size: 6,
                has_checksum: true,
                uses_xtea: false,
                uses_rsa: false,
            }
        );
    }

    #[test]
    fn protocol_settings_custom() {
        let cfg = ProtocolSettings {
            header_size: 4,
            has_checksum: false,
            uses_xtea: true,
            uses_rsa: false,
        };
        assert_eq!(cfg.header_size, 4);
        assert!(!cfg.has_checksum);
        assert!(cfg.uses_xtea);
        assert!(!cfg.uses_rsa);
    }
}
