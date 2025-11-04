use bytes::Buf;
use thiserror::Error;

/// Errors that can occur when decoding a packet from a byte buffer.
#[derive(Debug, Error)]
pub enum DecoderError {
    /// The buffer does not contain enough bytes to form a complete packet.
    ///
    /// This typically occurs when only part of the packet has been received.
    #[error("incomplete packet: expected {expected} bytes, but only {available} available")]
    Incomplete {
        /// Total number of bytes expected for the packet.
        expected: usize,
        /// Number of bytes currently available in the buffer.
        available: usize,
    },

    /// The packet contains invalid UTF-8 data when decoding a string.
    #[error("invalid UTF-8 data in packet")]
    InvalidUtf8(#[from] std::str::Utf8Error),
}

/// A trait for reading primitive types and strings from a byte buffer.
///
/// `Decoder` provides convenient methods for extracting booleans, integers,
/// strings, and raw byte slices from a packet buffer. It returns
/// a [`DecoderError`] if the buffer is incomplete or contains invalid data.
///
/// # Example
/// ```ignore
/// let mut buffer: &[u8] = &received_bytes;
/// let value: u16 = buffer.get_u16()?;
/// let flag: bool = buffer.get_bool()?;
/// let text: String = buffer.get_string()?;
/// ```
pub trait Decoder {
    fn get_bool(&mut self) -> Result<bool, DecoderError>;
    fn get_i8(&mut self) -> Result<i8, DecoderError>;
    fn get_u8(&mut self) -> Result<u8, DecoderError>;
    fn get_i16(&mut self) -> Result<i16, DecoderError>;
    fn get_u16(&mut self) -> Result<u16, DecoderError>;
    fn get_i32(&mut self) -> Result<i32, DecoderError>;
    fn get_u32(&mut self) -> Result<u32, DecoderError>;

    /// Reads a UTF-8 string prefixed with a 16-bit length field.
    fn get_string(&mut self) -> Result<String, DecoderError>;

    /// Returns all remaining bytes in the buffer.
    fn take_remaining(&mut self) -> &[u8];
}

impl Decoder for &mut &[u8] {
    fn get_bool(&mut self) -> Result<bool, DecoderError> {
        self.try_get_u8()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })
            .map(|value| value != 0)
    }

    fn get_i8(&mut self) -> Result<i8, DecoderError> {
        self.try_get_i8().map_err(|err| DecoderError::Incomplete {
            expected: err.requested,
            available: err.available,
        })
    }

    fn get_u8(&mut self) -> Result<u8, DecoderError> {
        self.try_get_u8().map_err(|err| DecoderError::Incomplete {
            expected: err.requested,
            available: err.available,
        })
    }

    fn get_i16(&mut self) -> Result<i16, DecoderError> {
        self.try_get_i16_le()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })
    }

    fn get_u16(&mut self) -> Result<u16, DecoderError> {
        self.try_get_u16_le()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })
    }

    fn get_i32(&mut self) -> Result<i32, DecoderError> {
        self.try_get_i32_le()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })
    }

    fn get_u32(&mut self) -> Result<u32, DecoderError> {
        self.try_get_u32_le()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })
    }

    fn get_string(&mut self) -> Result<String, DecoderError> {
        let length = self
            .try_get_u16_le()
            .map_err(|err| DecoderError::Incomplete {
                expected: err.requested,
                available: err.available,
            })? as usize;

        if self.len() < length {
            return Err(DecoderError::Incomplete {
                expected: length,
                available: self.len(),
            });
        }

        let (bytes, ..) = self.split_at(length);
        let str = std::str::from_utf8(bytes)?;
        self.advance(length);

        Ok(str.to_owned())
    }

    fn take_remaining(&mut self) -> &[u8] {
        let length = self.len();

        let (remaining, ..) = self.split_at(length);
        self.advance(length);

        remaining
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, DecoderError};

    #[test]
    fn get_bool_returns_true_and_false() {
        let data = vec![1, 0];

        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_bool().expect("Should get true");
        assert!(value, "Expected true");

        let value = data.get_bool().expect("Should get false");
        assert!(!value, "Expected false");
    }

    #[test]
    fn get_bool_returns_error_on_incomplete_buffer() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_bool().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 1, "Expected 1 byte for bool");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_i8_returns_value() {
        const VALUE: i8 = -42;

        let data = vec![VALUE as u8];
        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_i8().expect("Should get i8");
        assert_eq!(value, VALUE, "Value should match");
    }

    #[test]
    fn get_i8_returns_error_on_incomplete_buffer() {
        let data = Vec::new();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_i8().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 1, "Expected 1 byte for i8");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_u8_returns_value() {
        const VALUE: u8 = 200;

        let data = vec![VALUE];
        let mut data: &mut &[u8] = &mut data.as_slice();

        let val = data.get_u8().expect("Should get u8");
        assert_eq!(val, VALUE, "Value should match");
    }

    #[test]
    fn get_u8_returns_error_on_incomplete_buffer() {
        let data = Vec::new();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_u8().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 1, "Expected 1 byte for u8");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_i16_returns_value() {
        const VALUE: i16 = -12345;

        let data = VALUE.to_le_bytes().to_vec();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_i16().expect("Should get i16");
        assert_eq!(value, VALUE, "Value should match");
    }

    #[test]
    fn get_i16_returns_error_on_incomplete_buffer() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_i16().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 2, "Expected 2 bytes for i16");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_u16_returns_value() {
        const VALUE: u16 = 54321;

        let data = VALUE.to_le_bytes().to_vec();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_u16().expect("Should get u16");
        assert_eq!(value, VALUE, "Value should match");
    }

    #[test]
    fn get_u16_returns_error_on_incomplete_buffer() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_u16().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 2, "Expected 2 bytes for u16");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_i32_returns_value() {
        const VALUE: i32 = -987654321;

        let data = VALUE.to_le_bytes().to_vec();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_i32().expect("Should get i32");
        assert_eq!(value, VALUE, "Value should match");
    }

    #[test]
    fn get_i32_returns_error_on_incomplete_buffer() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_i32().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 4, "Expected 4 bytes for i32");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_u32_returns_value() {
        const VALUE: u32 = 1234567890;

        let data = VALUE.to_le_bytes().to_vec();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let value = data.get_u32().expect("Should get u32");
        assert_eq!(value, VALUE, "Value should match");
    }

    #[test]
    fn get_u32_returns_error_on_incomplete_buffer() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_u32().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 4, "Expected 4 bytes for u32");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn get_string_returns_valid_string() {
        const VALUE: &str = "test string";

        let mut datac = Vec::new();
        datac.extend_from_slice(&(VALUE.len() as u16).to_le_bytes());
        datac.extend_from_slice(VALUE.as_bytes());

        let mut data: &mut &[u8] = &mut datac.as_slice();

        let value = data.get_string().expect("Should get string");
        assert_eq!(value, VALUE, "String should match");
    }

    #[test]
    fn get_string_incomplete_length() {
        let data = Vec::new();

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_string().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, 2, "Expected 2 bytes for string length");
            assert_eq!(available, 0, "No bytes available");
        } else {
            panic!("Unexpected error: {:?}", err);
        }
    }

    #[test]
    fn get_string_incomplete_data() {
        const LENGTH: u16 = 5;

        let mut data = Vec::new();
        data.extend_from_slice(&LENGTH.to_le_bytes());
        data.extend_from_slice(b"abc");

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_string().expect_err("Expected incomplete error");
        if let DecoderError::Incomplete {
            expected,
            available,
        } = err
        {
            assert_eq!(expected, LENGTH as usize, "Expected string length");
            assert_eq!(available, 3, "Bytes available");
        } else {
            panic!("Unexpected error: {:?}", err);
        }
    }

    #[test]
    fn get_string_invalid_utf8() {
        const LENGTH: u16 = 2;

        let mut data = Vec::new();
        data.extend_from_slice(&LENGTH.to_le_bytes());
        data.extend_from_slice(&[0xff, 0xff]);

        let mut data: &mut &[u8] = &mut data.as_slice();

        let err = data.get_string().expect_err("Expected invalid UTF8 error");
        assert!(
            matches!(err, DecoderError::InvalidUtf8(..)),
            "Should be InvalidUtf8"
        );
    }

    #[test]
    fn take_remaining_returns_all_bytes_and_empty_buffer() {
        const DATA: [u8; 4] = [1, 2, 3, 4];

        let data = DATA.to_vec();
        let mut data: &mut &[u8] = &mut data.as_slice();

        let remaining = data.take_remaining();
        assert_eq!(remaining, &DATA, "Remaining data should match");
        assert_eq!(data.len(), 0, "Buffer should be empty after take_remaining");
    }

    #[test]
    fn decode_all_types_in_sequence() {
        const BOOL_TRUE: u8 = 1;
        const I8_NEGATIVE_42: i8 = -42;
        const U8_200: u8 = 200;
        const I16_NEGATIVE_12345: i16 = -12345;
        const U16_54321: u16 = 54321;
        const I32_NEGATIVE_987654321: i32 = -987654321;
        const U32_1234567890: u32 = 1234567890;
        const STRING_LEN: u16 = 5;
        const STRING: &str = "hello";

        let mut bytes = Vec::new();
        bytes.push(BOOL_TRUE);
        bytes.push(I8_NEGATIVE_42 as u8);
        bytes.push(U8_200);
        bytes.extend_from_slice(&I16_NEGATIVE_12345.to_le_bytes());
        bytes.extend_from_slice(&U16_54321.to_le_bytes());
        bytes.extend_from_slice(&I32_NEGATIVE_987654321.to_le_bytes());
        bytes.extend_from_slice(&U32_1234567890.to_le_bytes());
        bytes.extend_from_slice(&STRING_LEN.to_le_bytes());
        bytes.extend_from_slice(STRING.as_bytes());

        let mut buf: &mut &[u8] = &mut bytes.as_slice();

        assert!(buf.get_bool().expect("Should get bool"));
        assert_eq!(buf.get_i8().expect("Should get i8"), I8_NEGATIVE_42);
        assert_eq!(buf.get_u8().expect("Should get u8"), U8_200);
        assert_eq!(buf.get_i16().expect("Should get i16"), I16_NEGATIVE_12345);
        assert_eq!(buf.get_u16().expect("Should get u16"), U16_54321);
        assert_eq!(
            buf.get_i32().expect("Should get i32"),
            I32_NEGATIVE_987654321
        );
        assert_eq!(buf.get_u32().expect("Should get u32"), U32_1234567890);
        assert_eq!(buf.get_string().expect("Should get string"), STRING);
        assert_eq!(buf.len(), 0, "Buffer should be empty");
    }
}
