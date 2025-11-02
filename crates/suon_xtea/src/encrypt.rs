use crate::{XTEA_BLOCK_SIZE, XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey};
use bytes::{Bytes, BytesMut};

/// Encrypts data using the XTEA algorithm with a 128-bit key.
///
/// This function performs block encryption in-place, padding the plaintext with zeros
/// so that its length is a multiple of 8 bytes (the XTEA block size).
/// Each 8-byte block is split into two 32-bit words, undergoes 32 Feistel rounds,
/// and the resulting ciphertext is returned as an immutable [`Bytes`] buffer.
///
/// # Parameters
/// - `plaintext`: The raw data to be encrypted.
/// - `key`: A 128-bit encryption key [`XTEAKey`].
///
/// # Returns
/// Encrypted data as [`Bytes`], padded to a multiple of 8 bytes if necessary.
pub fn encrypt(plaintext: &[u8], key: &XTEAKey) -> Bytes {
    // Create a mutable buffer from the plaintext for padding.
    let mut padded_plaintext = BytesMut::from(plaintext);

    // Calculate padding to reach the next multiple of the block size.
    let padding_len =
        (XTEA_BLOCK_SIZE - (padded_plaintext.len() % XTEA_BLOCK_SIZE)) % XTEA_BLOCK_SIZE;

    // Pad with zeros if needed.
    if padding_len > 0 {
        padded_plaintext.extend(vec![0u8; padding_len]);
    }

    // Prepare buffer for ciphertext of the same size.
    let mut ciphertext = BytesMut::with_capacity(padded_plaintext.len());

    // Process each 8-byte block.
    for block in padded_plaintext.chunks(XTEA_BLOCK_SIZE) {
        // Copy block into fixed-size array.
        let mut block_bytes = [0u8; XTEA_BLOCK_SIZE];
        block_bytes.copy_from_slice(block);

        // Interpret as two 32-bit words in little-endian.
        let mut v0 = u32::from_le_bytes(block_bytes[0..4].try_into().unwrap());
        let mut v1 = u32::from_le_bytes(block_bytes[4..8].try_into().unwrap());

        // Initialize sum for key schedule.
        let mut sum: u32 = 0;

        // Perform 32 rounds of XTEA encryption.
        for _ in 0..XTEA_NUM_ROUNDS {
            v0 = v0.wrapping_add(
                ((v1 << 4) ^ (v1 >> 5)).wrapping_add(v1)
                    ^ (sum.wrapping_add(key[(sum & 3) as usize])),
            );
            sum = sum.wrapping_add(XTEA_DELTA);
            v1 = v1.wrapping_add(
                ((v0 << 4) ^ (v0 >> 5)).wrapping_add(v0)
                    ^ (sum.wrapping_add(key[((sum >> 11) & 3) as usize])),
            );
        }

        // Append encrypted words to ciphertext.
        ciphertext.extend_from_slice(&v0.to_le_bytes());
        ciphertext.extend_from_slice(&v1.to_le_bytes());
    }

    // Convert final buffer into immutable Bytes and return.
    ciphertext.freeze()
}
