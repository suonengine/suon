use bytes::{Bytes, BytesMut};
use thiserror::Error;

use crate::{XTEA_BLOCK_SIZE, XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey};

/// Errors that can occur during XTEA encryption and decryption.
#[derive(Debug, Error)]
pub enum XTEADecryptError {
    /// Occurs when the input data length is not a multiple of 8 bytes.
    #[error("Data length must be a multiple of 8 bytes")]
    InvalidBlockSize,

    /// Occurs when the inner length field of the message indicates a size
    /// that is larger than the actual buffer length.
    #[error("Inner length ({inner_length}) is larger than buffer length ({buffer_length})")]
    InnerLengthTooLarge {
        /// The inner length read from the message.
        inner_length: usize,
        /// The total available buffer length.
        buffer_length: usize,
    },

    /// Occurs when converting bytes to a `u32` fails.
    #[error("Failed to convert bytes to u32")]
    InvalidBytes,
}

/// Decrypts data encrypted with the XTEA algorithm using a 128-bit key.
///
/// This function processes ciphertext in 8-byte blocks and performs
/// the standard 32-round XTEA decryption. It also validates the
/// "inner length" field stored in the first two bytes of the decrypted data.
///
/// # Parameters
/// - `ciphertext`: The encrypted data to be decrypted. Must be a multiple of 8 bytes.
/// - `key`: The 128-bit key used for decryption.
///
/// # Returns
/// Returns a [`Bytes`] object containing the decrypted payload,
/// or an [`XTEADecryptError`] if the ciphertext is invalid.
///
/// # Errors
/// - [`XTEADecryptError::InvalidBlockSize`]: If the input is not a multiple of 8 bytes.
/// - [`XTEADecryptError::InvalidBytes`]: If conversion of bytes to 32-bit words fails.
/// - [`XTEADecryptError::InnerLengthTooLarge`]: If the payload length stored in the
///   first two bytes exceeds the available buffer size.
pub fn decrypt(ciphertext: &[u8], key: &XTEAKey) -> Result<Bytes, XTEADecryptError> {
    // Ensure the ciphertext length is a multiple of the XTEA block size.
    if ciphertext.len() % XTEA_BLOCK_SIZE != 0 {
        return Err(XTEADecryptError::InvalidBlockSize);
    }

    // Allocate a buffer for the decrypted output.
    let mut decrypted = BytesMut::with_capacity(ciphertext.len());

    // Process each 8-byte block individually.
    for block in ciphertext.chunks(XTEA_BLOCK_SIZE) {
        // Convert the block slice into a fixed-size array for safe processing.
        let block_bytes: [u8; XTEA_BLOCK_SIZE] = block
            .try_into()
            .map_err(|_| XTEADecryptError::InvalidBytes)?;

        // Split the block into two 32-bit words (little-endian).
        let mut word_left = u32::from_le_bytes(
            block_bytes[0..4]
                .try_into()
                .map_err(|_| XTEADecryptError::InvalidBytes)?,
        );

        let mut word_right = u32::from_le_bytes(
            block_bytes[4..8]
                .try_into()
                .map_err(|_| XTEADecryptError::InvalidBytes)?,
        );

        // Initialize the running sum for decryption (sum starts at delta * 32 rounds).
        let mut sum = XTEA_DELTA.wrapping_mul(XTEA_NUM_ROUNDS as u32);

        // Perform XTEA decryption rounds.
        for _ in 0..XTEA_NUM_ROUNDS {
            // Decrypt the right word using the left word and the key schedule.
            word_right = word_right.wrapping_sub(
                ((word_left << 4 ^ word_left >> 5).wrapping_add(word_left))
                    ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize])),
            );

            // Decrement the sum for the next decryption step.
            sum = sum.wrapping_sub(XTEA_DELTA);

            // Decrypt the left word using the updated right word.
            word_left = word_left.wrapping_sub(
                ((word_right << 4 ^ word_right >> 5).wrapping_add(word_right))
                    ^ (sum.wrapping_add(key[(sum & 3) as usize])),
            );
        }

        // Append the decrypted 64-bit block to the output buffer.
        decrypted.extend_from_slice(&word_left.to_le_bytes());
        decrypted.extend_from_slice(&word_right.to_le_bytes());
    }

    // Ensure that there are at least 2 bytes to read the inner payload length.
    if decrypted.len() < 2 {
        return Err(XTEADecryptError::InnerLengthTooLarge {
            inner_length: 0,
            buffer_length: ciphertext.len(),
        });
    }

    // Read the inner length from the first 2 bytes (little-endian).
    let inner_length = u16::from_le_bytes(
        decrypted[0..2]
            .try_into()
            .map_err(|_| XTEADecryptError::InvalidBytes)?,
    ) as usize;

    // Validate that the inner length does not exceed the total decrypted buffer.
    if inner_length + 2 > decrypted.len() {
        return Err(XTEADecryptError::InnerLengthTooLarge {
            inner_length,
            buffer_length: ciphertext.len(),
        });
    }

    // Keep only the meaningful payload (inner length + 2 bytes of header).
    decrypted.truncate(inner_length + 2);

    Ok(decrypted.freeze())
}
