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

    /// Generates a fixed test key used for encryption and decryption validation.
    const SAMPLE_KEY: XTEAKey = [0xA56BABCD, 0x00000000, 0xFFFFFFFF, 0x12345678];

    /// Helper to expand the sample key into round keys.
    fn expanded_sample_key() -> Vec<u32> {
        expand_key(&SAMPLE_KEY).to_vec()
    }

    #[test]
    fn test_expand_key_produces_valid_round_keys() {
        // Expand the key and verify correct round key generation
        let round_keys = expanded_sample_key();

        // Each XTEA round generates two subkeys (sum and difference phases)
        assert_eq!(round_keys.len(), XTEA_NUM_ROUNDS * 2);

        // Ensure at least one of the keys is non-zero
        assert!(
            round_keys.iter().any(|&k| k != 0),
            "Expanded key should not contain only zero values"
        );
    }

    #[test]
    fn test_encrypt_and_decrypt_roundtrip() {
        // Plaintext message for testing roundtrip correctness
        const MESSAGE: &[u8] = b"Suon Engine!";

        // Prefix the message with its 2-byte inner length
        let inner_len = MESSAGE.len() as u16;
        let mut data = inner_len.to_le_bytes().to_vec();
        data.extend_from_slice(MESSAGE);

        // Encrypt the data and decrypt back
        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let decrypted = decrypt(&ciphertext, &SAMPLE_KEY)
            .expect("Decryption should succeed for valid roundtrip");

        // The decrypted data must match the original input
        assert_eq!(decrypted, Bytes::from(data));
    }

    #[test]
    fn test_decrypt_rejects_invalid_block_size() {
        // Data with a non-8-byte-aligned length must fail
        const INVALID_DATA: &[u8] = &[1, 2, 3, 4, 5];

        let result = decrypt(INVALID_DATA, &SAMPLE_KEY);
        assert!(
            matches!(result, Err(XTEADecryptError::InvalidBlockSize)),
            "Expected InvalidBlockSize error for unaligned input"
        );
    }

    #[test]
    fn test_decrypt_rejects_inner_length_too_large() {
        // Declared inner length exceeds actual available bytes
        const DECLARED_LENGTH: u16 = 10;

        let mut data = DECLARED_LENGTH.to_le_bytes().to_vec();
        data.extend_from_slice(&[0u8; 2]);

        // Pad to nearest 8-byte boundary
        let padding = (8 - (data.len() % 8)) % 8;
        data.extend(vec![0u8; padding]);

        // Encrypt → Decrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let err =
            decrypt(&ciphertext, &SAMPLE_KEY).expect_err("Expected InnerLengthTooLarge error");

        // Validate the error variant and its fields
        match err {
            XTEADecryptError::InnerLengthTooLarge { inner_length, .. } => {
                assert_eq!(inner_length, DECLARED_LENGTH as usize);
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn test_encrypt_adds_padding_to_unaligned_input() {
        // 7-byte input should be padded to 8 bytes for block alignment
        const MESSAGE: &[u8] = b"1234567";

        let ciphertext = encrypt(MESSAGE, &SAMPLE_KEY);

        // XTEA requires data to be multiple of 8 bytes
        assert_eq!(ciphertext.len() % 8, 0);
    }

    #[test]
    fn test_decrypt_rejects_too_short_for_declared_length() {
        // Declared length larger than available payload
        const DECLARED_LENGTH: u16 = 20;

        let mut data = DECLARED_LENGTH.to_le_bytes().to_vec();
        data.extend_from_slice(&[0u8; 5]);

        // Pad to 8-byte alignment
        let padding = (8 - (data.len() % 8)) % 8;
        data.extend(vec![0u8; padding]);

        // Encrypt → Decrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let err =
            decrypt(&ciphertext, &SAMPLE_KEY).expect_err("Expected InnerLengthTooLarge error");

        match err {
            XTEADecryptError::InnerLengthTooLarge { inner_length, .. } => {
                assert_eq!(inner_length, DECLARED_LENGTH as usize);
            }
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }

    #[test]
    fn test_decrypt_valid_exact_inner_length() {
        // Message where declared inner length matches the actual payload
        const MESSAGE: &[u8] = b"ABCDEFGH";

        let inner_len = MESSAGE.len() as u16;
        let mut data = inner_len.to_le_bytes().to_vec();
        data.extend_from_slice(MESSAGE);

        // Encrypt → Decrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let decrypted = decrypt(&ciphertext, &SAMPLE_KEY)
            .expect("Decryption should succeed for exact-length message");

        // The decrypted result must match the input
        assert_eq!(decrypted, Bytes::from(data));
    }

    #[test]
    fn test_encrypt_and_decrypt_aligned_input() {
        // Input already 8-byte aligned → no extra padding should be needed
        const PAYLOAD: &[u8] = b"12345678";

        let inner_len = PAYLOAD.len() as u16;
        let mut data = inner_len.to_le_bytes().to_vec();
        data.extend_from_slice(PAYLOAD);

        // Encrypt → Decrypt
        let ciphertext = encrypt(&data, &SAMPLE_KEY);
        let decrypted =
            decrypt(&ciphertext, &SAMPLE_KEY).expect("Decryption should succeed for aligned input");

        // Validate decrypted content and ciphertext alignment
        assert_eq!(decrypted, Bytes::from(data));
        assert_eq!(ciphertext.len() % 8, 0);
    }

    #[test]
    fn test_decrypt_fails_on_too_small_input() {
        // Input shorter than a single XTEA block
        const INVALID_INPUT: &[u8] = &[1, 2, 3, 4];

        let result = decrypt(INVALID_INPUT, &SAMPLE_KEY);
        assert!(
            matches!(result, Err(XTEADecryptError::InvalidBlockSize)),
            "Expected InvalidBlockSize error for too-small input"
        );
    }
}
