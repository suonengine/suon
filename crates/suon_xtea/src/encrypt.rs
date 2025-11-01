use bytes::{Bytes, BytesMut};

use crate::{XTEA_BLOCK_SIZE, XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey};

/// Encrypts data using the XTEA algorithm with a 128-bit key.
///
/// This function performs block encryption in-place using the
/// standard XTEA (eXtended Tiny Encryption Algorithm) routine.
///
/// - Input data (`plaintext`) is automatically padded with zero bytes
///   so that its total length becomes a multiple of 8 bytes (the XTEA block size).
/// - Each 8-byte block is split into two 32-bit words, which undergo
///   32 Feistel-style encryption rounds.
/// - The function returns the resulting ciphertext as an immutable [`Bytes`] object.
///
/// # Parameters
/// - `plaintext`: The raw data to be encrypted.
/// - `key`: A 128-bit encryption key represented as [`XTEAKey`].
///
/// # Returns
/// The encrypted bytes, padded as necessary, wrapped in a [`Bytes`] buffer.
pub fn encrypt(plaintext: &[u8], key: &XTEAKey) -> Bytes {
    // Copy the plaintext into a mutable buffer so we can pad it if necessary.
    let mut padded_plaintext = BytesMut::from(plaintext);

    // Compute how many padding bytes are needed so that the total length
    // is a multiple of the 8-byte XTEA block size.
    let padding_len =
        (XTEA_BLOCK_SIZE - (padded_plaintext.len() % XTEA_BLOCK_SIZE)) % XTEA_BLOCK_SIZE;

    // Append zero padding if required.
    if padding_len > 0 {
        padded_plaintext.extend(vec![0u8; padding_len]);
    }

    // Pre-allocate the ciphertext buffer with the same total size as the padded plaintext.
    let mut ciphertext = BytesMut::with_capacity(padded_plaintext.len());

    // Process the plaintext in 8-byte (64-bit) chunks.
    for block in padded_plaintext.chunks(XTEA_BLOCK_SIZE) {
        // Copy the current block into a fixed-size 8-byte array.
        let mut block_bytes = [0u8; XTEA_BLOCK_SIZE];
        block_bytes.copy_from_slice(block);

        // Interpret the block as two 32-bit words in little-endian order.
        let mut word_left =
            u32::from_le_bytes(block_bytes[0..4].try_into().expect("slice must be 4 bytes"));
        let mut word_right =
            u32::from_le_bytes(block_bytes[4..8].try_into().expect("slice must be 4 bytes"));

        // Initialize the running sum used in XTEA’s key schedule.
        let mut sum: u32 = 0;

        // Perform 32 rounds of XTEA encryption.
        // Each round updates both halves (left and right) of the block
        // using bitwise shifts, XORs, additions, and key-dependent constants.
        for _ in 0..XTEA_NUM_ROUNDS {
            // Update the left word using the right word and part of the key.
            word_left = word_left.wrapping_add(
                ((word_right << 4 ^ word_right >> 5).wrapping_add(word_right))
                    ^ (sum.wrapping_add(key[(sum & 3) as usize])),
            );

            // Increment the round sum constant by the delta value (derived from the golden ratio).
            sum = sum.wrapping_add(XTEA_DELTA);

            // Update the right word using the left word and another part of the key.
            word_right = word_right.wrapping_add(
                ((word_left << 4 ^ word_left >> 5).wrapping_add(word_left))
                    ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize])),
            );
        }

        // Write the resulting encrypted 64-bit block into the ciphertext buffer.
        ciphertext.extend_from_slice(&word_left.to_le_bytes());
        ciphertext.extend_from_slice(&word_right.to_le_bytes());
    }

    // Return the final ciphertext as an immutable Bytes buffer.
    ciphertext.freeze()
}
