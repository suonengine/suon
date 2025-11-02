mod decrypt;
mod encrypt;
mod expand_key;

pub use decrypt::{XTEADecryptError, decrypt};
pub use encrypt::encrypt;
pub use expand_key::expand_key;

/// Represents a 128-bit XTEA key composed of four 32-bit words (4 × u32 = 16 bytes).
///
/// Each 32-bit word contributes to the overall 128-bit key used during
/// encryption and decryption rounds. This type ensures a consistent
/// and well-defined key structure across all XTEA operations.
pub type XTEAKey = [u32; 4];

/// Represents the fully expanded round key schedule used internally by XTEA.
///
/// Each XTEA round requires two 32-bit subkeys — one for the sum phase and
/// another for the difference phase — resulting in a total of
/// `XTEA_NUM_ROUNDS * 2` 32-bit round keys.
///
/// This precomputed key schedule improves performance by avoiding redundant
/// per-round key calculations.
pub(crate) type XTEARoundKeys = [u32; XTEA_NUM_ROUNDS * 2];

/// The delta constant used by the XTEA algorithm to adjust the running sum per round.
///
/// This value, derived from the golden ratio, ensures a good distribution of bits
/// across rounds and is fundamental to XTEA’s diffusion properties.
/// Each encryption or decryption round increments or decrements the sum by this delta.
pub(crate) const XTEA_DELTA: u32 = 0x9E3779B9;

/// Defines the number of encryption/decryption rounds executed per XTEA block.
///
/// The standard XTEA configuration uses 32 rounds, providing a strong balance
/// between security and performance. Increasing the number of rounds may enhance
/// security but at a computational cost.
pub(crate) const XTEA_NUM_ROUNDS: usize = 32;

/// The fixed size (in bytes) of a single XTEA encryption block.
///
/// XTEA operates exclusively on 64-bit blocks (8 bytes). Any input that is not
/// a multiple of this size must be padded before encryption and truncated after
/// decryption. This constant provides clarity and prevents accidental misuse
/// when handling raw buffers.
pub(crate) const XTEA_BLOCK_SIZE: usize = 8;

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    // Sample key used for encryption and decryption validation
    const SAMPLE_KEY: XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];

    // Helper function to expand the sample key into round keys
    fn get_expanded_keys() -> Vec<u32> {
        expand_key(&SAMPLE_KEY).to_vec()
    }

    #[test]
    fn test_expand_key_produces_valid_round_keys() {
        // Expand the key and verify the correct number of round keys
        let round_keys = get_expanded_keys();

        // Each round produces two subkeys
        assert_eq!(
            round_keys.len(),
            XTEA_NUM_ROUNDS * 2,
            "Incorrect number of round keys generated"
        );

        // Check that at least one key is non-zero, indicating proper expansion
        assert!(
            round_keys.iter().any(|&k| k != 0),
            "Expanded keys should contain non-zero values"
        );
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Sample message to test encryption and decryption cycle
        const MESSAGE: &[u8] = b"Sample Message";

        // Prefix with message length (little-endian)
        let length_prefix = (MESSAGE.len() as u16).to_le_bytes();
        let mut data = length_prefix.to_vec();
        data.extend_from_slice(MESSAGE);

        // Encrypt the data
        let ciphertext = encrypt(&data, &SAMPLE_KEY);

        // Decrypt back
        let decrypted = decrypt(&ciphertext, &SAMPLE_KEY)
            .expect("Decryption should succeed for valid ciphertext");

        // Confirm the decrypted data matches the original
        assert_eq!(decrypted, Bytes::from(data));
    }

    #[test]
    fn test_decrypt_rejects_invalid_block_size() {
        // Data not aligned to block size (8 bytes)
        const INVALID_DATA: &[u8] = &[1, 2, 3, 4, 5];

        let result = decrypt(INVALID_DATA, &SAMPLE_KEY);
        assert!(
            matches!(result, Err(XTEADecryptError::InvalidBlockSize)),
            "Expected InvalidBlockSize error for misaligned input"
        );
    }

    #[test]
    fn test_decrypt_rejects_inner_length_exceeds_payload() {
        // Declared inner length larger than actual payload
        const DECLARED_LENGTH: u16 = 10;

        let mut data = DECLARED_LENGTH.to_le_bytes().to_vec();
        // Less data than declared length
        data.extend_from_slice(&[0u8; 2]);

        // Pad to multiple of block size
        let padding = (8 - (data.len() % 8)) % 8;
        data.extend(vec![0u8; padding]);

        // Encrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);

        // Attempt to decrypt, expecting an error
        let err =
            decrypt(&ciphertext, &SAMPLE_KEY).expect_err("Expected InnerLengthTooLarge error");
        if let XTEADecryptError::InnerLengthTooLarge { inner_length, .. } = err {
            assert_eq!(inner_length, DECLARED_LENGTH as usize);
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn test_encrypt_adds_padding_for_unaligned_input() {
        // Input of 7 bytes, should be padded to 8 bytes
        const MESSAGE: &[u8] = b"1234567";

        let ciphertext = encrypt(MESSAGE, &SAMPLE_KEY);

        // Ciphertext length should be multiple of block size
        assert_eq!(
            ciphertext.len() % 8,
            0,
            "Ciphertext should be aligned to 8 bytes"
        );
    }

    #[test]
    fn test_decrypt_rejects_too_short_payload_for_declared_length() {
        // Declared length larger than available payload
        const DECLARED_LENGTH: u16 = 20;

        let mut data = DECLARED_LENGTH.to_le_bytes().to_vec();
        // Payload shorter than declared length
        data.extend_from_slice(&[0u8; 5]);

        // Pad to multiple of block size
        let padding = (8 - (data.len() % 8)) % 8;
        data.extend(vec![0u8; padding]);

        // Encrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);

        // Decrypt expecting an error
        let err =
            decrypt(&ciphertext, &SAMPLE_KEY).expect_err("Expected InnerLengthTooLarge error");
        if let XTEADecryptError::InnerLengthTooLarge { inner_length, .. } = err {
            assert_eq!(inner_length, DECLARED_LENGTH as usize);
        } else {
            panic!("Unexpected error variant: {:?}", err);
        }
    }

    #[test]
    fn test_decrypt_valid_exact_inner_length() {
        // Message where declared length matches actual payload
        const MESSAGE: &[u8] = b"ABCDEFGH";

        let inner_length = MESSAGE.len() as u16;
        let mut data = inner_length.to_le_bytes().to_vec();
        data.extend_from_slice(MESSAGE);

        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let decrypted = decrypt(&ciphertext, &SAMPLE_KEY)
            .expect("Decryption should succeed with exact inner length");

        assert_eq!(decrypted, Bytes::from(data));
    }

    #[test]
    fn test_encrypt_and_decrypt_aligned_input() {
        // Input already exactly 8 bytes, no padding needed
        const PAYLOAD: &[u8] = b"12345678";

        let inner_len = PAYLOAD.len() as u16;
        let mut data = inner_len.to_le_bytes().to_vec();
        data.extend_from_slice(PAYLOAD);

        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let decrypted = decrypt(&ciphertext, &SAMPLE_KEY).expect("Decryption should succeed");

        assert_eq!(decrypted, Bytes::from(data));
        assert_eq!(
            ciphertext.len() % 8,
            0,
            "Ciphertext should be aligned to 8 bytes"
        );
    }

    #[test]
    fn test_decrypt_fails_on_too_small_input() {
        // Input shorter than one block
        const INVALID_INPUT: &[u8] = &[1, 2, 3, 4];

        let result = decrypt(INVALID_INPUT, &SAMPLE_KEY);
        assert!(
            matches!(result, Err(XTEADecryptError::InvalidBlockSize)),
            "Expected InvalidBlockSize error for too small input"
        );
    }
}
