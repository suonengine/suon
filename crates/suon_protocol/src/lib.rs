#![warn(missing_docs)]

//! Shared wire-format codecs for the Suon protocol.
//!
//! All multi-byte integer types are encoded in **little-endian** byte order,
//! strings are length-prefixed with a `u16` little-endian length, and every
//! packet begins with a single-byte opcode (`PACKET_KIND_SIZE`).
//!
//! # Examples
//! ```
//! use suon_protocol::prelude::*;
//!
//! let encoded = Encoder::new().put_u16(7).put_str("suon").into_bytes();
//! let mut slice = encoded.as_ref();
//!
//! assert_eq!((&mut slice).get_u16().unwrap(), 7);
//! assert_eq!((&mut slice).get_string().unwrap(), "suon");
//! ```

mod packets;

/// Convenience module that re-exports the most common types from this crate.
pub mod prelude {
    pub use crate::packets::{
        PACKET_KIND_SIZE,
        decoder::{Decoder, DecoderError},
        encoder::Encoder,
    };

    /// Re-export of [`bytes::Bytes`] so that downstream crates do not need
    /// to add `bytes` as a direct dependency when working with packets.
    pub use bytes::Bytes;
}

#[cfg(test)]
mod tests {
    use crate::packets::{decoder::Decoder, encoder::Encoder};

    #[test]
    fn should_expose_codec_modules_from_crate_root() {
        let module_name = std::any::type_name::<Encoder>();

        assert!(
            module_name.contains("packets::encoder::Encoder"),
            "The crate should keep the encoder module available through the packets namespace"
        );
    }

    #[test]
    fn should_expose_protocol_codec_api_through_prelude() {
        use crate::prelude::*;

        fn assert_decodable<T: Decoder>() {}

        let _ = std::mem::size_of::<DecoderError>();
        let _ = std::mem::size_of::<Encoder>();

        assert_decodable::<&mut &[u8]>();

        assert_eq!(PACKET_KIND_SIZE, 1);
    }

    mod roundtrip {
        use super::*;

        #[test]
        fn bool_roundtrip() {
            let bytes = Encoder::new().put_bool(true).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            let result = decoder.get_bool().unwrap();
            assert!(result);
        }

        #[test]
        fn i8_roundtrip() {
            let value: i8 = -42;
            let bytes = Encoder::new().put_i8(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_i8().unwrap(), value);
        }

        #[test]
        fn u8_roundtrip() {
            let value: u8 = 200;
            let bytes = Encoder::new().put_u8(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_u8().unwrap(), value);
        }

        #[test]
        fn i16_roundtrip() {
            let value: i16 = -12345;
            let bytes = Encoder::new().put_i16(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_i16().unwrap(), value);
        }

        #[test]
        fn u16_roundtrip() {
            let value: u16 = 54321;
            let bytes = Encoder::new().put_u16(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_u16().unwrap(), value);
        }

        #[test]
        fn i32_roundtrip() {
            let value: i32 = -987654321;
            let bytes = Encoder::new().put_i32(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_i32().unwrap(), value);
        }

        #[test]
        fn u32_roundtrip() {
            let value: u32 = 1234567890;
            let bytes = Encoder::new().put_u32(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_u32().unwrap(), value);
        }

        #[test]
        fn i64_roundtrip() {
            let value: i64 = -9876543210;
            let bytes = Encoder::new().put_i64(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_i64().unwrap(), value);
        }

        #[test]
        fn i64_min_roundtrip() {
            let bytes = Encoder::new().put_i64(i64::MIN).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_i64().unwrap(), i64::MIN);
        }

        #[test]
        fn u64_roundtrip() {
            let value: u64 = 9876543210;
            let bytes = Encoder::new().put_u64(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_u64().unwrap(), value);
        }

        #[test]
        fn u64_max_roundtrip() {
            let bytes = Encoder::new().put_u64(u64::MAX).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_u64().unwrap(), u64::MAX);
        }

        #[test]
        fn raw_slice_roundtrip() {
            let payload: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];
            let bytes = Encoder::new().put_slice(payload).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.take_remaining(), payload);
        }

        #[test]
        fn string_roundtrip() {
            let value = "hello suon";
            let bytes = Encoder::new().put_str(value).into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;
            assert_eq!(decoder.get_string().unwrap(), value);
        }

        #[test]
        fn mixed_roundtrip() {
            let bytes = Encoder::new()
                .put_bool(true)
                .put_i8(-8)
                .put_u8(8)
                .put_i16(-16)
                .put_u16(16)
                .put_i32(-32)
                .put_u32(32)
                .put_i64(-64)
                .put_u64(64)
                .put_str("fim")
                .into_bytes();
            let mut buf: &[u8] = &bytes;
            let mut decoder: &mut &[u8] = &mut buf;

            assert!(decoder.get_bool().unwrap());
            assert_eq!(decoder.get_i8().unwrap(), -8);
            assert_eq!(decoder.get_u8().unwrap(), 8);
            assert_eq!(decoder.get_i16().unwrap(), -16);
            assert_eq!(decoder.get_u16().unwrap(), 16);
            assert_eq!(decoder.get_i32().unwrap(), -32);
            assert_eq!(decoder.get_u32().unwrap(), 32);
            assert_eq!(decoder.get_i64().unwrap(), -64);
            assert_eq!(decoder.get_u64().unwrap(), 64);
            assert_eq!(decoder.get_string().unwrap(), "fim");
            assert_eq!(decoder.remaining(), 0, "All bytes should be consumed");
        }
    }
}
