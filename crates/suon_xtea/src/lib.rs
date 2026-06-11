//! XTEA (eXtended Tiny Encryption Algorithm) — 64-bit Feistel block cipher.
//!
//! # Algorithm overview
//!
//! XTEA operates on 8-byte (64-bit) blocks using a 128-bit key split into
//! four 32-bit words. Each block is split into two 32-bit halves (`left`
//! and `right`) and processed through 32 rounds of a Feistel network.
//!
//! Encrypt round:
//! ```text
//! left  += mix(right) XOR (key[ sum & 3 ] + sum)
//! sum   += DELTA
//! right += mix(left)  XOR (key[(sum >> 11) & 3] + sum)
//! ```
//!
//! Decrypt is the inverse: same operations in reverse order with subtraction.
//!
//! DELTA = 0x9E37_79B9 is derived from the golden ratio: (√5 − 1) × 2³¹.
//!
//! # Key expansion
//!
//! The round keys can be precomputed once via [`expand()`] and reused across
//! multiple [`encrypt()`] / [`decrypt()`] calls for the same key.
//!
//! # Errors
//!
//! Returns [`XteaError::InvalidDataLength`] if `data.len()` is not a multiple
//! of 8 bytes.

use std::fmt;

/// Golden ratio constant used to derive per-round key material.
/// Each round adds DELTA to a running sum, which selects different key words
/// and provides round-specific key material.
const DELTA: u32 = 0x9E37_79B9;

/// Number of Feistel rounds. Total expanded key entries = `ROUNDS * 2 = 64`.
const ROUNDS: usize = 32;

/// Block size in bytes. XTEA operates on 64-bit (8-byte) blocks.
const BLOCK_SIZE: usize = 8;

/// Half-block size in bytes. Each block is split into two 32-bit halves.
const HALF_BLOCK: usize = BLOCK_SIZE / 2;

/// A 128-bit XTEA key stored as four 32-bit little-endian words.
pub type Key = [u32; 4];

/// Precomputed round keys for XTEA.
///
/// Interleaved format: `[left_key_0, right_key_0, left_key_1, right_key_1, ...]`
/// Produced by [`expand()`] and consumed by [`encrypt()`] / [`decrypt()`].
pub type ExpandedKey = [u32; ROUNDS * 2];

/// Errors returned by XTEA operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XteaError {
    /// The input data length is not a multiple of 8 bytes.
    InvalidDataLength(usize),
}

impl fmt::Display for XteaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XteaError::InvalidDataLength(len) => {
                write!(
                    f,
                    "XTEA data length {len} is not a multiple of {BLOCK_SIZE}"
                )
            }
        }
    }
}

impl std::error::Error for XteaError {}

/// XTEA non-linear mixing function.
///
/// Shifts `v` left by 4 bits and right by 5 bits, combines them with XOR,
/// then adds the original `v` back (all wrapping). This creates the diffusion
/// property essential for the Feistel network's security.
#[inline(always)]
fn mix(v: u32) -> u32 {
    ((v << 4) ^ (v >> 5)).wrapping_add(v)
}

/// Precomputes all 64 round keys from a 128-bit key.
///
/// The expanded key can be reused across multiple [`encrypt()`] / [`decrypt()`]
/// calls with the same key, avoiding redundant computation.
///
/// The output layout is interleaved:
/// `expanded[2*i]`   = left round key for round i
/// `expanded[2*i+1]` = right round key for round i
///
/// This function is `const`, so the expanded key can be computed at compile time
/// when the 128-bit key is known upfront.
#[inline(always)]
pub const fn expand(key: &Key) -> ExpandedKey {
    let mut round_keys = [0u32; ROUNDS * 2];

    // Running sum that advances by DELTA each round.
    // Used both to scramble the key selection (via `sum & 3` and `(sum >> 11) & 3`)
    // and as additive key material (via `key[...].wrapping_add(sum)`).
    let mut sum = 0u32;
    let mut key_index = 0;

    while key_index < ROUNDS * 2 {
        // Left half key: select key word based on low 2 bits of sum.
        round_keys[key_index] = key[(sum & 3) as usize].wrapping_add(sum);

        // Advance sum by DELTA for the right half key.
        sum = sum.wrapping_add(DELTA);

        // Right half key: select key word based on bits 11-12 of the new sum.
        round_keys[key_index + 1] = key[((sum >> 11) & 3) as usize].wrapping_add(sum);

        key_index += 2;
    }

    round_keys
}

/// Encrypts `data` in-place with XTEA using precomputed round keys.
///
/// For each round, all data blocks are processed sequentially before moving
/// to the next round. This keeps the round keys hot in cache and maximizes
/// instruction-level parallelism across independent blocks.
///
/// # Errors
///
/// Returns [`XteaError::InvalidDataLength`] if `data.len()` is not a multiple of 8.
pub fn encrypt(data: &mut [u8], expanded: &ExpandedKey) -> Result<(), XteaError> {
    let data_len = data.len();

    // Reject data that is not aligned to the 8-byte block boundary.
    if !data_len.is_multiple_of(BLOCK_SIZE) {
        return Err(XteaError::InvalidDataLength(data_len));
    }

    // Iterate over the 32 round pairs (64 entries, step 2).
    let mut key_index = 0;
    while key_index < ROUNDS * 2 {
        // Load the left and right round keys for this Feistel round.
        let left_key = expanded[key_index];
        let right_key = expanded[key_index + 1];

        // Apply this round's transformation to every block in the buffer.
        for block in data.chunks_exact_mut(BLOCK_SIZE) {
            // Read the two 32-bit halves from the block (little-endian).
            let left = u32::from_le_bytes([block[0], block[1], block[2], block[3]]);
            let right = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

            // Feistel round: scramble the opposite half, XOR with key material,
            // then add (wrapping) to the current half.
            let new_left = left.wrapping_add(mix(right) ^ left_key);
            let new_right = right.wrapping_add(mix(new_left) ^ right_key);

            // Write the updated halves back to the block (little-endian).
            block[..HALF_BLOCK].copy_from_slice(&new_left.to_le_bytes());
            block[HALF_BLOCK..BLOCK_SIZE].copy_from_slice(&new_right.to_le_bytes());
        }

        key_index += 2;
    }

    Ok(())
}

/// Decrypts `data` in-place with XTEA using precomputed round keys.
///
/// Processes rounds in reverse order, streaming through all blocks per round
/// for optimal cache behavior and instruction-level parallelism.
///
/// # Errors
///
/// Returns [`XteaError::InvalidDataLength`] if `data.len()` is not a multiple of 8.
pub fn decrypt(data: &mut [u8], expanded: &ExpandedKey) -> Result<(), XteaError> {
    let data_len = data.len();

    // Reject data that is not aligned to the 8-byte block boundary.
    if !data_len.is_multiple_of(BLOCK_SIZE) {
        return Err(XteaError::InvalidDataLength(data_len));
    }

    // Empty data trivially decrypts to empty data.
    if data_len == 0 {
        return Ok(());
    }

    // Start from the last round pair and work backwards.
    let mut key_index = ROUNDS * 2 - 1;
    loop {
        // Load the right and left round keys for this round.
        // Note: right_key is loaded first because decrypt reverses the
        // order of operations within each Feistel round.
        let right_key = expanded[key_index];
        let left_key = expanded[key_index - 1];

        // Apply this round's inverse transformation to every block.
        for block in data.chunks_exact_mut(BLOCK_SIZE) {
            // Read the two 32-bit halves from the block (little-endian).
            let left = u32::from_le_bytes([block[0], block[1], block[2], block[3]]);
            let right = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);

            // Inverse Feistel round: subtract (wrapping) instead of adding.
            // The right key is applied first (mirroring the encrypt order).
            let new_right = right.wrapping_sub(mix(left) ^ right_key);
            let new_left = left.wrapping_sub(mix(new_right) ^ left_key);

            // Write the decrypted halves back to the block (little-endian).
            block[..HALF_BLOCK].copy_from_slice(&new_left.to_le_bytes());
            block[HALF_BLOCK..BLOCK_SIZE].copy_from_slice(&new_right.to_le_bytes());
        }

        // Stop after processing the first round pair (indices 0 and 1).
        if key_index == 1 {
            break;
        }

        key_index -= 2;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0xABu8; 24];
        let original_buffer = buffer.clone();
        encrypt(&mut buffer, &expanded_keys).expect("encrypt should succeed for 24-byte input");
        assert_ne!(buffer, original_buffer);
        decrypt(&mut buffer, &expanded_keys).expect("decrypt should succeed for valid ciphertext");
        assert_eq!(buffer, original_buffer);
    }

    #[test]
    fn encrypt_decrypt_empty_blocks() {
        let key = [1, 2, 3, 4];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0u8; 16];
        let original_buffer = buffer.clone();
        encrypt(&mut buffer, &expanded_keys).expect("encrypt should succeed for 16 zero bytes");
        decrypt(&mut buffer, &expanded_keys)
            .expect("decrypt should succeed for 16 encrypted bytes");
        assert_eq!(buffer, original_buffer);
    }

    #[test]
    fn encrypt_deterministic() {
        let key = [0, 0, 0, 0];
        let expanded_keys = expand(&key);
        let mut plaintext_a = vec![0u8; 8];
        encrypt(&mut plaintext_a, &expanded_keys).expect("first encrypt should succeed");
        let mut plaintext_b = vec![0u8; 8];
        encrypt(&mut plaintext_b, &expanded_keys).expect("second encrypt should succeed");
        assert_eq!(
            plaintext_a, plaintext_b,
            "same key + same plaintext must produce same ciphertext"
        );
    }

    #[test]
    fn different_keys_produce_different_output() {
        let key1 = [0, 0, 0, 0];
        let key2 = [1, 2, 3, 4];
        let expanded_keys_1 = expand(&key1);
        let expanded_keys_2 = expand(&key2);
        let mut plaintext_a = vec![0xFFu8; 8];
        let mut plaintext_b = vec![0xFFu8; 8];
        encrypt(&mut plaintext_a, &expanded_keys_1).expect("encrypt with key1 should succeed");
        encrypt(&mut plaintext_b, &expanded_keys_2).expect("encrypt with key2 should succeed");
        assert_ne!(
            plaintext_a, plaintext_b,
            "different keys must produce different ciphertext"
        );
    }

    #[test]
    fn encrypt_decrypt_multiple_blocks() {
        let key = [0xDE, 0xAD, 0xBE, 0xEF];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0xABu8; 64];
        let original_buffer = buffer.clone();
        encrypt(&mut buffer, &expanded_keys).expect("encrypt should succeed for 64 bytes");
        decrypt(&mut buffer, &expanded_keys).expect("decrypt should succeed for 64 bytes");
        assert_eq!(buffer, original_buffer);
    }

    #[test]
    fn encrypted_data_differs_from_plaintext() {
        let key = [0x12, 0x34, 0x56, 0x78];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0u8; 8];
        encrypt(&mut buffer, &expanded_keys).expect("encrypt should succeed for 8 bytes");
        assert_ne!(
            buffer,
            vec![0u8; 8],
            "ciphertext must differ from plaintext"
        );
    }

    #[test]
    fn encrypt_then_decrypt_large_data() {
        let key = [1, 2, 3, 4];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0x42u8; 1024];
        let original_buffer = buffer.clone();
        encrypt(&mut buffer, &expanded_keys).expect("encrypt should succeed for 1 KiB");
        decrypt(&mut buffer, &expanded_keys).expect("decrypt should succeed for 1 KiB");
        assert_eq!(buffer, original_buffer);
    }

    #[test]
    fn zero_key_encrypts_to_non_zero() {
        let key = [0, 0, 0, 0];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0u8; 8];
        encrypt(&mut buffer, &expanded_keys).expect("encrypt with zero key should succeed");
        assert!(
            buffer.iter().any(|&byte| byte != 0),
            "zero-key encryption must produce non-zero ciphertext"
        );
        decrypt(&mut buffer, &expanded_keys)
            .expect("decrypt after zero-key encrypt should succeed");
        assert_eq!(buffer, vec![0u8; 8]);
    }

    #[test]
    fn known_answer_test_zero_key() {
        let key = [0x00000000, 0x00000000, 0x00000000, 0x00000000];
        let expanded_keys = expand(&key);
        let mut buffer = vec![0u8; 8];
        encrypt(&mut buffer, &expanded_keys).expect("encrypt for zero-key KAT should succeed");
        assert_eq!(
            buffer,
            vec![0xd8, 0xd4, 0xe9, 0xde, 0xd9, 0x1e, 0x13, 0xf7],
            "zero-key KAT ciphertext mismatch"
        );
        decrypt(&mut buffer, &expanded_keys).expect("decrypt for zero-key KAT should succeed");
        assert_eq!(
            buffer,
            vec![0u8; 8],
            "zero-key KAT decrypt must restore zeros"
        );
    }

    #[test]
    fn known_answer_test_nonzero_key() {
        let key = [0x12345678, 0x9ABCDEF0, 0x0FEDCBA9, 0x87654321];
        let expanded_keys = expand(&key);
        let mut buffer = b"ABCDEFGH".to_vec();
        encrypt(&mut buffer, &expanded_keys).expect("encrypt for non-zero KAT should succeed");
        assert_eq!(
            buffer,
            vec![0x5c, 0x25, 0x02, 0xff, 0xad, 0x19, 0x2a, 0xd0],
            "non-zero key KAT ciphertext mismatch"
        );
        decrypt(&mut buffer, &expanded_keys).expect("decrypt for non-zero KAT should succeed");
        assert_eq!(
            buffer, b"ABCDEFGH",
            "non-zero KAT decrypt must restore original"
        );
    }

    #[test]
    fn encrypt_rejects_non_multiple_of_8() {
        let key = [0; 4];
        let expanded_keys = expand(&key);
        let result = encrypt(&mut [0u8; 7], &expanded_keys);
        assert!(matches!(result, Err(XteaError::InvalidDataLength(7))));
    }

    #[test]
    fn decrypt_rejects_non_multiple_of_8() {
        let key = [0; 4];
        let expanded_keys = expand(&key);
        let result = decrypt(&mut [0u8; 7], &expanded_keys);
        assert!(matches!(result, Err(XteaError::InvalidDataLength(7))));
    }

    #[test]
    fn encrypt_decrypt_empty_data() {
        let key = [0xDEAD, 0xBEEF, 0xCAFE, 0xBABE];
        let expanded_keys = expand(&key);
        let mut data: Vec<u8> = vec![];
        encrypt(&mut data, &expanded_keys).expect("encrypt on empty data should succeed");
        assert_eq!(data.len(), 0, "encrypt must not change empty data length");
        decrypt(&mut data, &expanded_keys).expect("decrypt on empty data should succeed");
        assert_eq!(data.len(), 0, "decrypt must not change empty data length");
    }

    #[test]
    fn expand_key_reuse() {
        let key = [0x01, 0x23, 0x45, 0x67];
        let expanded_keys = expand(&key);

        let mut buffer_a = vec![0xABu8; 16];
        let mut buffer_b = vec![0xCDu8; 16];
        let original_a = buffer_a.clone();
        let original_b = buffer_b.clone();

        encrypt(&mut buffer_a, &expanded_keys)
            .expect("first encrypt with reused key should succeed");
        encrypt(&mut buffer_b, &expanded_keys)
            .expect("second encrypt with reused key should succeed");
        assert_ne!(buffer_a, original_a, "first buffer must be encrypted");
        assert_ne!(buffer_b, original_b, "second buffer must be encrypted");

        decrypt(&mut buffer_a, &expanded_keys)
            .expect("first decrypt with reused key should succeed");
        decrypt(&mut buffer_b, &expanded_keys)
            .expect("second decrypt with reused key should succeed");
        assert_eq!(
            buffer_a, original_a,
            "first buffer roundtrip must restore original"
        );
        assert_eq!(
            buffer_b, original_b,
            "second buffer roundtrip must restore original"
        );
    }

    #[test]
    fn expand_const_fn() {
        // Asserts that `expand` can be evaluated at compile time.
        const EXPANDED: ExpandedKey = expand(&[0; 4]);
        let expanded = expand(&[0; 4]);
        assert_eq!(EXPANDED, expanded);
    }

    #[test]
    fn expand_key_one_shot_convenience() {
        let key = [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210];
        let mut data = vec![0xABu8; 24];
        let original_data = data.clone();

        encrypt(&mut data, &expand(&key)).expect("one-shot encrypt should succeed");
        assert_ne!(
            data, original_data,
            "one-shot encrypt must produce ciphertext"
        );
        decrypt(&mut data, &expand(&key)).expect("one-shot decrypt should succeed");
        assert_eq!(
            data, original_data,
            "one-shot roundtrip must restore original"
        );
    }
}
