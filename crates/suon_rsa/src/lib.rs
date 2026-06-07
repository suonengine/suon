//! RSA (Rivest–Shamir–Adleman) — asymmetric encryption cipher.
//!
//! # Algorithm overview
//!
//! RSA uses a key pair: a public key `(modulus, public_exponent)` for
//! encryption and a private key `(modulus, private_exponent)` for
//! decryption.  Both operations are modular exponentiation in the ring
//! of integers modulo `modulus`:
//!
//! ```text
//! Encryption:   ciphertext = plaintext^public_exponent  (mod modulus)
//! Decryption:   plaintext  = ciphertext^private_exponent (mod modulus)
//! ```
//!
//! The key components satisfy
//! `public_exponent · private_exponent ≡ 1 (mod φ(modulus))`, so
//! applying both operations in sequence recovers the original message
//! (provided the message is numerically smaller than `modulus`).
//!
//! This implementation performs **raw RSA** (no padding).  Padding must
//! be applied by the caller for security in production use.
//!
//! # Key loading
//!
//! Private keys are loaded from PEM-encoded PKCS#1 format via
//! [`load_pem()`].  The resulting [`Rsa`] struct holds the modulus `n`,
//! public exponent `e`, and private exponent `d` so it can be used for
//! both encryption and decryption.
//!
//! The PEM parsing pipeline:
//! 1. [`pem`] decodes the base64-encoded DER body.
//! 2. [`pkcs1`] parses the DER-encoded PKCS#1 `RSAPrivateKey` structure.
//! 3. The three big integers (modulus, public_exponent, private_exponent)
//!    are extracted into [`num_bigint_dig::BigUint`] values for fast
//!    modular exponentiation.
//!
//! # Block size
//!
//! Raw RSA operates on exactly `key_size` bytes at a time, where
//! `key_size = ceil(key_bits / 8)`.  For a 1024-bit key this is
//! 128 bytes.
//!
//! # Errors
//!
//! Returns [`RsaError::InvalidKey`] if the PEM data cannot be parsed or
//! if the data length does not match the key size.

use std::fmt;

use num_bigint_dig::BigUint;
use pkcs1::der::Decode;

/// Key size in bytes for a 1024-bit RSA key.  Useful for pre-allocating
/// buffers when the key size is known at compile time.
#[allow(dead_code)]
const KEY_SIZE_1024: usize = 128;

/// RSA key material extracted from a PEM-encoded private key.
///
/// Contains the modulus `modulus`, public exponent `public_exponent`,
/// and private exponent `private_exponent` needed for raw RSA encryption
/// and decryption.
///
/// Created via [`load_pem()`].
pub struct Rsa {
    /// RSA modulus `modulus` (product of two large primes).
    modulus: BigUint,

    /// Public exponent `public_exponent` (typically 65537).
    public_exponent: BigUint,

    /// Private exponent `private_exponent`.
    ///
    /// Satisfies `public_exponent * private_exponent ≡ 1 (mod φ(modulus))`.
    private_exponent: BigUint,
}

/// Errors returned by RSA operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsaError {
    /// The PEM data could not be parsed as a valid RSA private key, or the
    /// input data length does not match the key size.
    InvalidKey,
}

impl fmt::Display for RsaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RsaError::InvalidKey => write!(f, "invalid RSA key or data length"),
        }
    }
}

impl std::error::Error for RsaError {}

/// Loads an RSA private key from PEM-encoded PKCS#1 data.
///
/// The PEM must be in traditional PKCS#1 format:
///
/// ```text
/// -----BEGIN RSA PRIVATE KEY-----
/// MII...
/// -----END RSA PRIVATE KEY-----
/// ```
///
/// # Pipeline
///
/// 1. The PEM wrapper is stripped and base64-decoded by the [`pem`] crate.
/// 2. The resulting DER is parsed as a PKCS#1 `RSAPrivateKey` structure
///    by the [`pkcs1`] crate.
/// 3. The three big integers (modulus, public_exponent, private_exponent)
///    are converted to [`BigUint`] values and stored in the returned
///    [`Rsa`] struct.
///
/// # Errors
///
/// Returns [`RsaError::InvalidKey`] if the PEM data is malformed or does
/// not contain a valid RSA private key.
pub fn load_pem(pem_data: &str) -> Result<Rsa, RsaError> {
    // Strip the PEM armour and base64-decode the body.
    let der_bytes = pem::parse(pem_data).map_err(|_| RsaError::InvalidKey)?;

    // Parse the DER-encoded PKCS#1 RSAPrivateKey structure.
    let key_info =
        pkcs1::RsaPrivateKey::from_der(der_bytes.contents()).map_err(|_| RsaError::InvalidKey)?;

    Ok(Rsa {
        // BigUint::from_bytes_be strips any leading zero byte that
        // ASN.1 INTEGER encoding may have added, so the conversion is
        // lossless.
        modulus: BigUint::from_bytes_be(key_info.modulus.as_bytes()),
        public_exponent: BigUint::from_bytes_be(key_info.public_exponent.as_bytes()),
        private_exponent: BigUint::from_bytes_be(key_info.private_exponent.as_bytes()),
    })
}

/// Decrypts `data` in-place with raw RSA using the private exponent.
///
/// Performs `plaintext = ciphertext^private_exponent (mod modulus)`.
/// The input must be exactly `key_size` bytes long.  The result
/// overwrites the input in place.
///
/// # Errors
///
/// Returns [`RsaError::InvalidKey`] if `data.len()` does not match the
/// key size in bytes.
pub fn decrypt(key: &Rsa, data: &mut [u8]) -> Result<(), RsaError> {
    let key_size = key_size_bytes(&key.modulus);

    // Reject data that does not match the exact key block size.
    if data.len() != key_size {
        return Err(RsaError::InvalidKey);
    }

    // Interpret the ciphertext as a big-endian integer.
    let ciphertext = BigUint::from_bytes_be(data);

    // Raw RSA decryption: plaintext = ciphertext^private_exponent mod modulus.
    let plaintext = ciphertext.modpow(&key.private_exponent, &key.modulus);

    // Write the result back, right-aligned in the buffer.
    write_bigint_be(&plaintext, data);

    Ok(())
}

/// Encrypts `data` in-place with raw RSA using the public exponent.
///
/// Performs `ciphertext = plaintext^public_exponent (mod modulus)`.
/// The input must be strictly less than `modulus` and exactly
/// `key_size` bytes long.  The result overwrites the input in place.
///
/// # Errors
///
/// Returns [`RsaError::InvalidKey`] if `data.len()` does not match the
/// key size in bytes.
pub fn encrypt(key: &Rsa, data: &mut [u8]) -> Result<(), RsaError> {
    let key_size = key_size_bytes(&key.modulus);

    // Reject data that does not match the exact key block size.
    if data.len() != key_size {
        return Err(RsaError::InvalidKey);
    }

    // Interpret the plaintext as a big-endian integer.
    let plaintext = BigUint::from_bytes_be(data);

    // Raw RSA encryption: ciphertext = plaintext^public_exponent mod modulus.
    let ciphertext = plaintext.modpow(&key.public_exponent, &key.modulus);

    // Write the result back, right-aligned in the buffer.
    write_bigint_be(&ciphertext, data);

    Ok(())
}

/// Returns the key size in bytes, rounded up to the nearest byte.
///
/// For a 1024-bit key this returns 128; for a 2048-bit key, 256.
fn key_size_bytes(modulus: &BigUint) -> usize {
    modulus.bits().div_ceil(8)
}

/// Writes a [`BigUint`] into a fixed-size byte slice, right-aligned
/// (i.e. padded with leading zeros if the value is shorter than the
/// output buffer).
///
/// This ensures the output always has exactly `output.len()` bytes so
/// the result can be used in-place by the caller.
fn write_bigint_be(value: &BigUint, output: &mut [u8]) {
    let value_bytes = value.to_bytes_be();
    let output_len = output.len();

    if value_bytes.len() >= output_len {
        // The serialised value is at least as large as the output
        // buffer.  Take only the trailing bytes — this handles the case
        // where BigUint::to_bytes_be emits a leading zero byte that
        // ASN.1 INTEGER encoding would require but our fixed-size
        // layout does not.
        output.copy_from_slice(&value_bytes[value_bytes.len() - output_len..]);
    } else {
        // The serialised value is shorter than the buffer; left-pad
        // with zeros so the result is right-aligned.
        let padding = output_len - value_bytes.len();
        output[..padding].fill(0);
        output[padding..].copy_from_slice(&value_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1024-bit RSA private key used in all tests.
    ///
    /// This is the same key used by the TFS test suite
    /// (`test_rsa.cpp`).  The modulus is 1024 bits and the public
    /// exponent is 65537.
    const RSA_PRIVATE_KEY_PEM: &str = concat!(
        "-----BEGIN RSA PRIVATE KEY-----\n",
        "MIICXAIBAAKBgQCbZGkDtFsHrJVlaNhzU71xZROd15QHA7A+bdB5OZZhtKg3qmBWHXzLlFL6AIBZ\n",
        "SQmIKrW8pYoaGzX4sQWbcrEhJhHGFSrT27PPvuetwUKnXT11lxUJwyHFwkpb1R/UYPAbThW+sN4Z\n",
        "MFKKXT8VwePL9cQB1nd+EKyqsz2+jVt/9QIDAQABAoGAQovTtTRtr3GnYRBvcaQxAvjIV9ZUnFRm\n",
        "C7Y3i1KwJhOZ3ozmSLrEEOLqTgoc7R+sJ1YzEiDKbbete11EC3gohlhW56ptj0WDf+7ptKOgqiEy\n",
        "Kh4qt1sYJeeGz4GiiooJoeKFGdtk/5uvMR6FDCv6H7ewigVswzf330Q3Ya7+jYECQQERBxsga6+5\n",
        "x6IofXyNF6QuMqvuiN/pUgaStUOdlnWBf/T4yUpKvNS1+I4iDzqGWOOSR6RsaYPYVhj9iRABoKyx\n",
        "AkEAkbNzB6vhLAWht4dUdGzaREF3p4SwNcu5bJRa/9wCLSHaS9JaTq4lljgVPp1zyXyJCSCWpFnl\n",
        "0WvK3Qf6nVBIhQJBANS7rK8+ONWQbxENdZaZ7Rrx8HUTwSOS/fwhsGWBbl1Qzhdq/6/sIfEHkfeH\n",
        "1hoH+IlpuPuf21MdAqvJt+cMwoECQF1LyBOYduYGcSgg6u5mKVldhm3pJCA+ZGxnjuGZEnet3qeA\n",
        "eb05++112fyvO85ABUun524z9lokKNFh45NKLjUCQGshzV43P+RioiBhtEpB/QFzijiS4L2HKNu1\n",
        "tdhudnUjWkaf6jJmQS/ppln0hhRMHlk9Vus/bPx7LtuDuo6VQDo=\n",
        "-----END RSA PRIVATE KEY-----\n",
    );

    #[test]
    fn load_pem_valid_key() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        assert_eq!(key.modulus.bits(), 1024, "key should be 1024-bit");
    }

    #[test]
    fn load_pem_invalid_key() {
        let result = load_pem("not a valid PEM string");
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // Use 0x78 ('x') which is < first byte of modulus (0x9b),
        // guaranteeing that the plaintext is numerically smaller than
        // the modulus.
        let mut buffer = vec![0x78u8; KEY_SIZE_1024];
        let original_buffer = buffer.clone();

        encrypt(&key, &mut buffer).expect("encrypt should succeed");
        assert_ne!(
            buffer, original_buffer,
            "ciphertext must differ from plaintext"
        );

        decrypt(&key, &mut buffer).expect("decrypt should succeed");
        assert_eq!(
            buffer, original_buffer,
            "decrypt must restore original plaintext"
        );
    }

    #[test]
    fn encrypt_deterministic() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        let mut first_buffer = vec![0x78u8; KEY_SIZE_1024];
        let mut second_buffer = vec![0x78u8; KEY_SIZE_1024];

        encrypt(&key, &mut first_buffer).expect("first encrypt should succeed");
        encrypt(&key, &mut second_buffer).expect("second encrypt should succeed");

        assert_eq!(
            first_buffer, second_buffer,
            "same plaintext must produce same ciphertext"
        );
    }

    #[test]
    fn known_answer_decrypt() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // Ciphertext obtained by encrypting 128 bytes of 0x78 with the
        // public key using raw RSA (no padding).  This vector is taken
        // directly from the TFS test_rsa.cpp test suite.
        let mut ciphertext: Vec<u8> = vec![
            0x72, 0x17, 0x59, 0x03, 0xe4, 0xe9, 0xf8, 0x51, 0xce, 0x44, 0x0f, 0x83, 0x35, 0xbf,
            0x65, 0xf0, 0x23, 0xe9, 0x80, 0xfc, 0x8c, 0x80, 0x43, 0x08, 0xa4, 0x0e, 0xd2, 0xc1,
            0x1d, 0x7d, 0x03, 0x38, 0xb0, 0x3b, 0x0b, 0xb6, 0xd1, 0xf9, 0xf4, 0x55, 0xdc, 0x71,
            0x12, 0xc2, 0x17, 0x92, 0xee, 0xd3, 0x22, 0xfa, 0xd4, 0x24, 0xd3, 0xd5, 0x05, 0x5d,
            0x38, 0x34, 0xd4, 0x12, 0xdf, 0x3b, 0x0d, 0xc5, 0xa8, 0x59, 0xe5, 0x9d, 0x1f, 0x92,
            0xb6, 0x3f, 0x54, 0x0a, 0xe0, 0x44, 0xeb, 0x6e, 0x55, 0x0a, 0x8e, 0xd0, 0xd1, 0xf7,
            0x84, 0x1d, 0x3c, 0x0b, 0xcc, 0x3e, 0x2b, 0x08, 0x83, 0x3d, 0xa7, 0x83, 0x67, 0xb8,
            0x3d, 0x49, 0xda, 0x13, 0xde, 0x41, 0x18, 0x7f, 0x42, 0xb2, 0x80, 0x8f, 0x9b, 0xe6,
            0xfe, 0x4b, 0xb7, 0xe2, 0xab, 0x98, 0x0f, 0x4a, 0xdd, 0x52, 0xe9, 0xb1, 0x5b, 0xef,
            0x25, 0x03,
        ];

        decrypt(&key, &mut ciphertext).expect("decrypt should succeed");

        let expected_plaintext = vec![0x78u8; KEY_SIZE_1024];
        assert_eq!(
            ciphertext, expected_plaintext,
            "decrypted data must match known answer"
        );
    }

    #[test]
    fn encrypt_rejects_wrong_size() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        let result = encrypt(&key, &mut [0u8; 64]);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn decrypt_rejects_wrong_size() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        let result = decrypt(&key, &mut [0u8; 64]);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn decrypt_rejects_empty_data() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        let result = decrypt(&key, &mut []);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }
}
