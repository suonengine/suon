use bytes::{BufMut, Bytes, BytesMut};

/// A helper struct for encoding primitive types and strings into a byte buffer,
/// suitable for sending as a network packet.
///
/// `Encoder` provides convenient methods for writing integers, booleans, strings,
/// and raw bytes into a contiguous buffer. After populating the buffer, it can be
/// finalized into an immutable [`Bytes`] instance for transmission.
///
/// # Example
/// ```ignore
/// let packet_bytes = Encoder::new()
///     .put_u8(42)
///     .put_str("Hello")
///     .finalize();
/// ```
pub struct Encoder {
    buffer: BytesMut,
}

impl Encoder {
    /// Default initial buffer size, in bytes.
    ///
    /// This small size is enough for typical initial packets and avoids excessive allocations.
    pub const INITIAL_CAPACITY: usize = 1024;

    /// Creates a new encoder with the default initial capacity.
    pub fn new() -> Encoder {
        Encoder {
            buffer: BytesMut::with_capacity(Self::INITIAL_CAPACITY),
        }
    }

    /// Creates a new encoder with a custom initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(capacity),
        }
    }

    /// Writes a boolean as a single byte (0 = false, 1 = true).
    pub fn put_bool(&mut self, value: bool) -> &mut Self {
        self.buffer.put_u8(value as u8);
        self
    }

    /// Writes a signed 8-bit integer.
    pub fn put_i8(&mut self, value: i8) -> &mut Self {
        self.buffer.put_i8(value);
        self
    }

    /// Writes an unsigned 8-bit integer.
    pub fn put_u8(&mut self, value: u8) -> &mut Self {
        self.buffer.put_u8(value);
        self
    }

    /// Writes a signed 16-bit integer in little-endian format.
    pub fn put_i16(&mut self, value: i16) -> &mut Self {
        self.buffer.put_i16_le(value);
        self
    }

    /// Writes an unsigned 16-bit integer in little-endian format.
    pub fn put_u16(&mut self, value: u16) -> &mut Self {
        self.buffer.put_u16_le(value);
        self
    }

    /// Writes a signed 32-bit integer in little-endian format.
    pub fn put_i32(&mut self, value: i32) -> &mut Self {
        self.buffer.put_i32_le(value);
        self
    }

    /// Writes an unsigned 32-bit integer in little-endian format.
    pub fn put_u32(&mut self, value: u32) -> &mut Self {
        self.buffer.put_u32_le(value);
        self
    }

    /// Writes a UTF-8 string with a 16-bit length prefix.
    ///
    /// The string is encoded as:
    /// 1. A 2-byte little-endian length field.
    /// 2. UTF-8 bytes of the string.
    pub fn put_str(&mut self, value: &str) -> &mut Self {
        let bytes = value.as_bytes();
        self.buffer.put_u16_le(bytes.len() as u16);
        self.buffer.put_slice(bytes);
        self
    }

    /// Writes a raw byte buffer into the encoder.
    pub fn put_bytes(&mut self, bytes: Bytes) -> &mut Self {
        self.buffer.put(bytes);
        self
    }

    /// Finalizes the buffer and returns an immutable [`Bytes`] instance suitable for sending.
    pub fn finalize(&mut self) -> Bytes {
        self.buffer.clone().freeze()
    }
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn encoder_put_u8_writes_expected_byte() {
        const VALUE: u8 = 42;

        let result = Encoder::new().put_u8(VALUE).finalize();

        assert_eq!(
            result.as_ref(),
            &[VALUE],
            "Encoder should write the u8 byte VALUE correctly"
        );
    }

    #[test]
    fn encoder_put_bool_writes_0_for_false_and_1_for_true() {
        const FALSE: bool = false;
        const TRUE: bool = true;

        let result = Encoder::new().put_bool(FALSE).put_bool(TRUE).finalize();

        assert_eq!(
            result.as_ref(),
            &[FALSE as u8, TRUE as u8],
            "Encoder should encode FALSE as 0 and TRUE as 1"
        );
    }

    #[test]
    fn encoder_put_i16_writes_little_endian_bytes() {
        const VALUE: i16 = 0x1234;

        let result = Encoder::new().put_i16(VALUE).finalize();

        const EXPECTED: [u8; 2] = [0x34, 0x12];
        assert_eq!(
            result.as_ref(),
            &EXPECTED,
            "Encoder should write i16 VALUE in little-endian order"
        );
    }

    #[test]
    fn encoder_put_u16_writes_little_endian_bytes() {
        const VALUE: u16 = 0xABCD;

        let result = Encoder::new().put_u16(VALUE).finalize();

        const EXPECTED: [u8; 2] = [0xCD, 0xAB];
        assert_eq!(
            result.as_ref(),
            &EXPECTED,
            "Encoder should write u16 VALUE in little-endian order"
        );
    }

    #[test]
    fn encoder_put_i32_and_u32_writes_little_endian_bytes() {
        const I32_VALUE: i32 = 0x12345678;
        const U32_VALUE: u32 = 0x90ABCDEF;

        let result = Encoder::new()
            .put_i32(I32_VALUE)
            .put_u32(U32_VALUE)
            .finalize();

        const EXPECTED: [u8; 8] = [0x78, 0x56, 0x34, 0x12, 0xEF, 0xCD, 0xAB, 0x90];
        assert_eq!(
            result.as_ref(),
            &EXPECTED,
            "Encoder should write I32_VALUE and U32_VALUE in little-endian order"
        );
    }

    #[test]
    fn encoder_put_str_writes_length_and_bytes() {
        const VALUE: &str = "AB";

        let result = Encoder::new().put_str(VALUE).finalize();

        const EXPECTED: [u8; 4] = [0x02, 0x00, 0x41, 0x42];
        assert_eq!(
            result.as_ref(),
            &EXPECTED,
            "Encoder should write string VALUE length as u16 LE followed by UTF-8 bytes"
        );
    }

    #[test]
    fn encoder_put_bytes_appends_raw_bytes() {
        const BYTES: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF];

        let result = Encoder::new()
            .put_bytes(Bytes::from_static(BYTES))
            .finalize();

        assert_eq!(
            result.as_ref(),
            BYTES,
            "Encoder should append raw BYTES correctly"
        );
    }

    #[test]
    fn encoder_with_capacity_initializes_correctly() {
        const CAPACITY: usize = 128;

        let encoder = Encoder::with_capacity(CAPACITY).finalize();

        assert!(
            encoder.is_empty(),
            "Encoder with CAPACITY should start with an empty buffer"
        );
    }

    #[test]
    fn default_encoder_is_equal_to_new() {
        let default_encoder = Encoder::default().finalize();
        let new_encoder = Encoder::new().finalize();

        assert_eq!(
            default_encoder.as_ref(),
            new_encoder.as_ref(),
            "Default encoder should produce the same initial buffer as Encoder::new"
        );
    }
}
