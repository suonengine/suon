use bytes::{Bytes, BytesMut};
use thiserror::Error;

use crate::{XTEA_BLOCK_SIZE, XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey};

/// Errors that can occur during XTEA decryption.
#[derive(Debug, Error)]
pub enum XTEADecryptError {
    /// Occurs when the input data length is not a multiple of 8 bytes.
    #[error("Data length must be a multiple of 8 bytes")]
    InvalidBlockSize,

    /// Occurs when the inner length field of the message indicates a size
    /// that is larger than the actual buffer length.
    #[error("Inner length ({inner_length}) is larger than buffer length ({buffer_length})")]
    InnerLengthTooLarge {
        inner_length: usize,
        buffer_length: usize,
    },

    /// Occurs when converting bytes to a `u32` fails.
    #[error("Failed to convert bytes to u32")]
    InvalidBytes,
}

/// Decrypts data encrypted with the XTEA algorithm using a 128-bit key.
///
/// Processes ciphertext in 8-byte blocks, performing the standard 32-round XTEA decryption.
/// Validates the "inner length" field stored in the first two bytes of the decrypted data.
///
/// # Parameters
/// - `ciphertext`: The encrypted data to be decrypted. Must be a multiple of 8 bytes.
/// - `key`: The 128-bit key used for decryption.
///
/// # Returns
/// - `Ok(Bytes)`: The decrypted payload, including the header and inner data.
/// - `Err(XTEADecryptError)`: If the ciphertext is invalid.
pub fn decrypt(ciphertext: &[u8], key: &XTEAKey) -> Result<Bytes, XTEADecryptError> {
    // Check if input length is a multiple of block size.
    if !ciphertext.len().is_multiple_of(XTEA_BLOCK_SIZE) {
        return Err(XTEADecryptError::InvalidBlockSize);
    }

    // Prepare buffer for decrypted data.
    let mut decrypted = BytesMut::with_capacity(ciphertext.len());

    // Process each 8-byte block.
    for block in ciphertext.chunks(XTEA_BLOCK_SIZE) {
        // Convert block slice to fixed-size array.
        let block_bytes: [u8; XTEA_BLOCK_SIZE] = block
            .try_into()
            .map_err(|_| XTEADecryptError::InvalidBytes)?;

        // Split into two 32-bit words (little-endian).
        let mut v0 = u32::from_le_bytes(block_bytes[0..4].try_into().unwrap());
        let mut v1 = u32::from_le_bytes(block_bytes[4..8].try_into().unwrap());

        // Initialize sum for decryption.
        let mut sum = XTEA_DELTA.wrapping_mul(XTEA_NUM_ROUNDS as u32);

        // Perform 32 decryption rounds.
        for _ in 0..XTEA_NUM_ROUNDS {
            v1 = v1.wrapping_sub(
                ((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)
                    ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize])),
            );
            sum = sum.wrapping_sub(XTEA_DELTA);
            v0 = v0.wrapping_sub(
                ((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)
                    ^ (sum.wrapping_add(key[(sum & 3) as usize])),
            );
        }

        // Append decrypted words to buffer.
        decrypted.extend_from_slice(&v0.to_le_bytes());
        decrypted.extend_from_slice(&v1.to_le_bytes());
    }

    // Ensure buffer has at least 2 bytes to read inner length.
    if decrypted.len() < 2 {
        return Err(XTEADecryptError::InnerLengthTooLarge {
            inner_length: 0,
            buffer_length: ciphertext.len(),
        });
    }

    // Read inner length (little-endian).
    let inner_length = u16::from_le_bytes(decrypted[0..2].try_into().unwrap()) as usize;

    // Validate inner length against total decrypted data.
    if inner_length + 2 > decrypted.len() {
        return Err(XTEADecryptError::InnerLengthTooLarge {
            inner_length,
            buffer_length: ciphertext.len(),
        });
    }

    // Keep only the relevant payload: header + inner data.
    decrypted.truncate(inner_length + 2);

    Ok(decrypted.freeze())
}
