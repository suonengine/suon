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
//! # Decryption with CRT
//!
//! Instead of computing `ciphertext^private_exponent mod modulus`
//! (a full-size modular exponentiation on the entire key), decryption
//! uses the **Chinese Remainder Theorem** (Garner's algorithm) to split
//! the work across the two prime factors `p` and `q`:
//!
//! ```text
//! m1 = ciphertext^dP mod p      (512-bit exponent → 512-bit modulus)
//! m2 = ciphertext^dQ mod q      (512-bit exponent → 512-bit modulus)
//!  h = qInv · (m1 − m2) mod p   (recombination step)
//!  m = m2 + q · h               (full-size result)
//! ```
//!
//! Each `modpow` operates on half the bits (512 vs 1024), yielding a
//! ~4× speed-up over the naive approach.
//!
//! # Key loading
//!
//! Private keys are loaded from PEM-encoded PKCS#1 format via
//! [`load_pem()`].  The resulting [`Rsa`] struct holds all CRT
//! parameters, so the Garner algorithm is available immediately.
//!
//! The PEM parsing pipeline:
//! 1. The PEM armour is stripped and the body is base64-decoded.
//! 2. The resulting DER is parsed as a PKCS#1 `RSAPrivateKey` structure.
//! 3. All big integers (modulus, exponents, primes, CRT coefficient)
//!    are extracted into [`num_bigint_dig::BigUint`] values.
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
use tracing::{info, trace};

/// Key size in bytes for a 1024-bit RSA key.  Useful for pre-allocating
/// buffers when the key size is known at compile time.
#[allow(dead_code)]
const KEY_SIZE_1024: usize = 128;

/// DER tag for a SEQUENCE (constructed, universal class).
const DER_TAG_SEQUENCE: u8 = 0x30;

/// DER tag for an INTEGER (primitive, universal class).
const DER_TAG_INTEGER: u8 = 0x02;

/// Flag bit that distinguishes long-form DER length encoding.
/// When set, the lower 7 bits indicate how many subsequent bytes hold
/// the actual length value.
const DER_LONG_FLAG: u8 = 0x80;

/// Mask used to extract the byte count from a long-form length octet.
const DER_LENGTH_MASK: u8 = 0x7F;

/// PKCS#1 version value for a two-prime RSA key (multi-prime not supported).
const TWO_PRIME_KEY_VERSION: u8 = 0x00;

/// Number of base64 characters in a single encoding group (3 bytes → 4 chars).
const GROUP_SIZE: usize = 4;

/// Number of output bytes produced by one base64 group (4 chars → 3 bytes).
const OUTPUT_SIZE: usize = 3;

/// RSA key material extracted from a PEM-encoded private key.
///
/// Contains the full CRT parameter set so that decryption runs the
/// Garner algorithm internally, which is ~4× faster than naive
/// `ciphertext^private_exponent mod modulus`.
///
/// Created via [`load_pem()`].
pub struct Rsa {
    /// RSA modulus `n` (product of two large primes).
    modulus: BigUint,

    /// Public exponent `e` (typically 65537).
    public_exponent: BigUint,

    /// Private exponent `d` satisfying `e · d ≡ 1 (mod φ(n))`.
    #[allow(dead_code)]
    private_exponent: BigUint,

    /// First prime factor `p` of the modulus.
    prime_p: BigUint,

    /// Second prime factor `q` of the modulus.
    prime_q: BigUint,

    /// CRT exponent for `p`: `dP = d mod (p − 1)`.
    ///
    /// Used in the first Garner step: `m1 = ciphertext^dP mod p`.
    exponent_dp: BigUint,

    /// CRT exponent for `q`: `dQ = d mod (q − 1)`.
    ///
    /// Used in the second Garner step: `m2 = ciphertext^dQ mod q`.
    exponent_dq: BigUint,

    /// CRT coefficient: `qInv = q^(−1) mod p`.
    ///
    /// Used in the Garner recombination step:
    /// `h = qInv · (m1 − m2) mod p`.
    coefficient: BigUint,

    /// Key size in bytes, cached to avoid recomputing it on every
    /// encrypt / decrypt call.
    key_size: usize,
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
/// All CRT parameters are extracted so that [`decrypt()`] can use the
/// Garner algorithm internally.
///
/// # Errors
///
/// Returns [`RsaError::InvalidKey`] if the PEM data is malformed or does
/// not contain a valid RSA private key.
pub fn load_pem(pem_data: &str) -> Result<Rsa, RsaError> {
    let der_bytes = pem_decode(pem_data)?;
    let mut offset = 0;

    // SEQUENCE { INTEGER version, INTEGER n, INTEGER e, INTEGER d,
    //            INTEGER p, INTEGER q, INTEGER dP, INTEGER dQ, INTEGER qInv }
    if der_bytes.get(offset) != Some(&DER_TAG_SEQUENCE) {
        return Err(RsaError::InvalidKey);
    }

    offset += 1;
    offset = skip_der_length(&der_bytes, offset)?;

    let version = read_der_integer(&der_bytes, &mut offset)?;
    if version != [TWO_PRIME_KEY_VERSION] {
        return Err(RsaError::InvalidKey);
    }

    let modulus = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let public_exponent = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let private_exponent = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let prime_p = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let prime_q = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let exponent_dp = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let exponent_dq = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);
    let coefficient = BigUint::from_bytes_be(read_der_integer(&der_bytes, &mut offset)?);

    let key_size = modulus.bits().div_ceil(8);

    info!(target: "Rsa",
        "RSA key loaded: {} bits, key_size={} bytes",
        key_size * 8,
        key_size
    );

    Ok(Rsa {
        key_size,
        modulus,
        public_exponent,
        private_exponent,
        prime_p,
        prime_q,
        exponent_dp,
        exponent_dq,
        coefficient,
    })
}

fn pem_decode(pem_data: &str) -> Result<Vec<u8>, RsaError> {
    let mut base64 = Vec::new();
    let mut in_body = false;

    for line in pem_data.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("-----BEGIN") {
            in_body = true;
            continue;
        }

        if trimmed.starts_with("-----END") {
            break;
        }

        if in_body {
            base64.extend_from_slice(trimmed.as_bytes());
        }
    }

    if base64.is_empty() {
        return Err(RsaError::InvalidKey);
    }

    base64_decode(&base64)
}

fn base64_decode(input: &[u8]) -> Result<Vec<u8>, RsaError> {
    let decode = |byte: u8| -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        }
    };

    if !input.len().is_multiple_of(GROUP_SIZE) {
        return Err(RsaError::InvalidKey);
    }

    let mut output = Vec::with_capacity(input.len() / GROUP_SIZE * OUTPUT_SIZE);

    for chunk in input.chunks(GROUP_SIZE) {
        let value_0 = decode(chunk[0]).ok_or(RsaError::InvalidKey)?;
        let value_1 = decode(chunk[1]).ok_or(RsaError::InvalidKey)?;
        let value_2 = decode(chunk[2]).ok_or(RsaError::InvalidKey)?;
        let value_3 = decode(chunk[3]).ok_or(RsaError::InvalidKey)?;

        let triple = (value_0 as u32) << 18
            | (value_1 as u32) << 12
            | (value_2 as u32) << 6
            | value_3 as u32;

        let padding = chunk.iter().filter(|&&byte| byte == b'=').count();

        output.push((triple >> 16) as u8);

        if padding < 2 {
            output.push((triple >> 8) as u8);
        }

        if padding < 1 {
            output.push(triple as u8);
        }
    }

    Ok(output)
}

fn skip_der_length(data: &[u8], offset: usize) -> Result<usize, RsaError> {
    let len_byte = data.get(offset).copied().ok_or(RsaError::InvalidKey)?;
    if len_byte < DER_LONG_FLAG {
        return Ok(offset + 1);
    }

    let byte_count = (len_byte & DER_LENGTH_MASK) as usize;
    if byte_count == 0 || offset + 1 + byte_count > data.len() {
        return Err(RsaError::InvalidKey);
    }

    Ok(offset + 1 + byte_count)
}

fn read_der_integer<'a>(data: &'a [u8], offset: &mut usize) -> Result<&'a [u8], RsaError> {
    if data.get(*offset) != Some(&DER_TAG_INTEGER) {
        return Err(RsaError::InvalidKey);
    }

    *offset += 1;

    let len_byte = data.get(*offset).copied().ok_or(RsaError::InvalidKey)?;
    let value_len: usize;
    if len_byte < DER_LONG_FLAG {
        value_len = len_byte as usize;
        *offset += 1;
    } else {
        let byte_count = (len_byte & DER_LENGTH_MASK) as usize;
        if byte_count == 0 || byte_count > 4 || *offset + 1 + byte_count > data.len() {
            return Err(RsaError::InvalidKey);
        }

        value_len = data[*offset + 1..*offset + 1 + byte_count]
            .iter()
            .fold(0usize, |acc, &b| acc * 256 + b as usize);
        *offset += 1 + byte_count;
    }

    if *offset + value_len > data.len() {
        return Err(RsaError::InvalidKey);
    }

    let value = &data[*offset..*offset + value_len];
    *offset += value_len;

    Ok(value)
}

/// Decrypts `data` in-place with raw RSA using the CRT-accelerated path.
///
/// Internally uses the Chinese Remainder Theorem (Garner's algorithm)
/// with the five CRT parameters stored in the [`Rsa`] key.  This is
/// ~4× faster than the naive `ciphertext^private_exponent mod modulus`.
///
/// Garner's algorithm:
/// ```text
/// m1 = ciphertext^dP mod p      ← exponentiation with half-size key
/// m2 = ciphertext^dQ mod q      ← exponentiation with half-size key
///  h = qInv · (m1 − m2) mod p   ← linear recombination
///  m = m2 + q · h               ← full-size result (< modulus)
/// ```
///
/// # Errors
///
/// Returns [`RsaError::InvalidKey`] if `data.len()` does not match the
/// key size in bytes.
pub fn decrypt(key: &Rsa, data: &mut [u8]) -> Result<(), RsaError> {
    if data.len() != key.key_size {
        trace!(target: "Rsa",
            "RSA decrypt: data length {} != key size {}",
            data.len(),
            key.key_size
        );
        return Err(RsaError::InvalidKey);
    }

    trace!(target: "Rsa", "RSA decrypt start: {} bytes", data.len());
    let ciphertext = BigUint::from_bytes_be(data);

    // CRT: m1 = c^dP mod p, m2 = c^dQ mod q.
    let result_prime_p = ciphertext.modpow(&key.exponent_dp, &key.prime_p);
    let result_prime_q = ciphertext.modpow(&key.exponent_dq, &key.prime_q);

    // CRT: h = qInv · (m1 − m2) mod p.  Add p when m1 < m2 (BigUint).
    // Reuse result_prime_p's allocation for diff.
    let mut diff = if result_prime_p >= result_prime_q {
        let mut d = result_prime_p;
        d -= &result_prime_q;
        d
    } else {
        let mut d = result_prime_p;
        d += &key.prime_p;
        d -= &result_prime_q;
        d
    };

    // factor = (diff * coefficient) % prime_p — reuses diff's allocation.
    diff *= &key.coefficient;
    diff %= &key.prime_p;

    // CRT: m = m2 + q · h (always < modulus).
    // Reuse diff's allocation for the product.
    let mut prod = diff;
    prod *= &key.prime_q;
    let plaintext = result_prime_q + &prod;

    write_bigint_be(&plaintext, data);
    trace!(target: "Rsa", "RSA decrypt done");
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
    if data.len() != key.key_size {
        trace!(target: "Rsa",
            "RSA encrypt: data length {} != key size {}",
            data.len(),
            key.key_size
        );
        return Err(RsaError::InvalidKey);
    }

    trace!(target: "Rsa", "RSA encrypt start: {} bytes", data.len());
    let plaintext = BigUint::from_bytes_be(data);

    // Raw RSA encryption: c = m^e mod n.
    let ciphertext = plaintext.modpow(&key.public_exponent, &key.modulus);

    write_bigint_be(&ciphertext, data);
    trace!(target: "Rsa", "RSA encrypt done");
    Ok(())
}

#[inline(always)]
fn write_bigint_be(value: &BigUint, output: &mut [u8]) {
    let out_len = output.len();
    let bits = value.bits();
    if bits == 0 {
        output.fill(0);
        return;
    }

    let limb = value.get_limb(0);
    let bpl = std::mem::size_of_val(&limb);
    let bits_per_limb = bpl * 8;
    let num_limbs = bits.div_ceil(bits_per_limb);
    let total_bytes = num_limbs * bpl;
    let pad = out_len - total_bytes;

    output[..pad].fill(0);

    for i in 0..num_limbs {
        let digit: u64 = value.get_limb(num_limbs - 1 - i);
        output[pad + i * bpl..pad + i * bpl + bpl].copy_from_slice(&digit.to_be_bytes()[8 - bpl..]);
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

    /// Second 1024-bit RSA private key used for key-comparison tests.
    ///
    /// Generated independently so its modulus and exponents differ
    /// from [`RSA_PRIVATE_KEY_PEM`].
    const RSA_PRIVATE_KEY_PEM_2: &str = concat!(
        "-----BEGIN RSA PRIVATE KEY-----\n",
        "MIICXAIBAAKBgQDModnN6z8dwGmZspvWppgdES/wucxfrXGbwIS7KoX8UHLulStK\n",
        "K1O7zaSBrvoc8bfW8Uz/l/u3ImA49VbXHEvdAxU0mfvvC1wCtEjitDPaQ7VVI9xq\n",
        "dUWZukr26owm79ZcJyLgQ6Hau+UPLPhIG15Vulm4dyPXs3sjxKxeMojTpQIDAQAB\n",
        "AoGADLVId3dSliBq7nafIvd5nuSAW6zOOmrlEU0lcRI0+/RrDtIIvDRwoMsmmj8p\n",
        "nT6NsjWOGJlxsm/aFe92kylYtKZ6Mx3uc9ljQRZf7RQc5jNH91Mmh4b/5x+n6KGY\n",
        "D8yoX4tqGJtT8972G4Csyw4zjQ1GYj0NDjCq8I9oRNYEEsECQQD8R6X6jr9sgA0J\n",
        "WJPEKqTOJo86Fk/ayWNfjz1xIxXSEOX4fRSwdkl0hdZ9ZDQN8INKHZvnspTRfleL\n",
        "kbxQxrGtAkEAz6ZVYE1/o3HBfBPxl7oP1YgRUnKH87ZS0AnTzCRrRMINZ8hjcy6j\n",
        "Fa7SnTF4ICKEA5SUGr7tOzu+3dkTdcQY2QJBAPUYyumVe+Z2tbOpyc3gvELIdYgy\n",
        "mxxtYc06RbBALPfskPCM3Off09eQG+Wwz13nmDYOdCRzfF/XxkgDq5gyofUCQB7x\n",
        "ZGuTYN/URcbdmfTILy/ctOgaVRQGKVUDAeK70phOanz6qYcyfe7vPEdcZdA0FIQM\n",
        "Ef3iUauv/YNFo9a6wBECQAvUgG1RHNaoqviDQvc8TfW1Mqqcs6YHgp37xf9BjLzD\n",
        "iVGgcCp5PJrPW506K72M57otY8dTjaiouMXCeOQaYz4=\n",
        "-----END RSA PRIVATE KEY-----\n",
    );

    /// PEM-encoded 1024-bit RSA public key (no private components).
    ///
    /// Parsing this with [`load_pem()`] must fail because it does not
    /// contain the private CRT parameters.
    const RSA_PUBLIC_KEY_PEM: &str = concat!(
        "-----BEGIN RSA PUBLIC KEY-----\n",
        "MIGJAoGBAMyh2c3rPx3AaZmym9ammB0RL/C5zF+tcZvAhLsqhfxQcu6VK0orU7vN\n",
        "pIGu+hzxt9bxTP+X+7ciYDj1VtccS90DFTSZ++8LXAK0SOK0M9pDtVUj3Gp1RZm6\n",
        "SvbqjCbv1lwnIuBDodq75Q8s+EgbXlW6Wbh3I9ezeyPErF4yiNOlAgMBAAE=\n",
        "-----END RSA PUBLIC KEY-----\n",
    );

    #[test]
    fn load_pem_valid_key() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        assert_eq!(key.modulus.bits(), 1024, "key should be 1024-bit");
        assert_eq!(key.key_size, 128, "cached key_size should be 128");
        assert!(key.prime_p.bits() > 0, "prime_p must exist");
        assert!(key.prime_q.bits() > 0, "prime_q must exist");
        assert!(
            key.modulus == &key.prime_p * &key.prime_q,
            "modulus must equal p * q"
        );
    }

    #[test]
    fn load_pem_invalid_key() {
        let result = load_pem("not a valid PEM string");
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn load_pem_public_key_rejected() {
        let result = load_pem(RSA_PUBLIC_KEY_PEM);
        assert!(
            matches!(result, Err(RsaError::InvalidKey)),
            "public key PEM must be rejected"
        );
    }

    #[test]
    fn load_pem_corrupted_base64_rejected() {
        let corrupted =
            "-----BEGIN RSA PRIVATE KEY-----\n!!!INVALID!!!\n-----END RSA PRIVATE KEY-----\n";
        let result = load_pem(corrupted);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn load_pem_non_rsa_der_rejected() {
        // A PEM with a valid DER structure that is NOT an RSA private key
        // (an empty SEQUENCE encoded in DER, wrapped in PEM armour).
        let non_rsa_pem = concat!(
            "-----BEGIN RSA PRIVATE KEY-----\n",
            "MAoGCCqGSIb3DQEBCw==\n",
            "-----END RSA PRIVATE KEY-----\n",
        );
        let result = load_pem(non_rsa_pem);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // Use 0x78 ('x') which is < first byte of modulus (0x9b),
        // guaranteeing that the plaintext is numerically smaller than
        // the modulus.
        let mut buffer = vec![0x78u8; key.key_size];
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
    fn encrypted_data_differs_from_plaintext() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // 0^e = 0 and 1^e = 1 in any modulus, so those plaintexts
        // encrypt to themselves in raw RSA.  Use a non-trivial value.
        let mut buffer = vec![0x02u8; key.key_size];
        encrypt(&key, &mut buffer).expect("encrypt should succeed");
        assert!(
            buffer.iter().any(|&byte| byte != 0x02),
            "ciphertext must differ from plaintext"
        );
    }

    #[test]
    fn encrypt_deterministic() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        let mut first_buffer = vec![0x78u8; key.key_size];
        let mut second_buffer = vec![0x78u8; key.key_size];

        encrypt(&key, &mut first_buffer).expect("first encrypt should succeed");
        encrypt(&key, &mut second_buffer).expect("second encrypt should succeed");

        assert_eq!(
            first_buffer, second_buffer,
            "same plaintext must produce same ciphertext"
        );
    }

    #[test]
    fn different_keys_produce_different_output() {
        let key1 = load_pem(RSA_PRIVATE_KEY_PEM).expect("first key should load");
        let key2 = load_pem(RSA_PRIVATE_KEY_PEM_2).expect("second key should load");

        // Both keys are 1024-bit with the same plaintext.
        assert_eq!(key1.key_size, key2.key_size);

        let mut ciphertext1 = vec![0x78u8; key1.key_size];
        let mut ciphertext2 = vec![0x78u8; key1.key_size];

        encrypt(&key1, &mut ciphertext1).expect("first encrypt should succeed");
        encrypt(&key2, &mut ciphertext2).expect("second encrypt should succeed");

        assert_ne!(
            ciphertext1, ciphertext2,
            "different keys must produce different ciphertext"
        );
    }

    #[test]
    fn encrypt_decrypt_large_data() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // 1 KiB of data, processed 128 bytes at a time.
        let total_size = 1024;
        let mut buffer = vec![0x42u8; total_size];
        let original_buffer = buffer.clone();

        for chunk in buffer.chunks_mut(key.key_size) {
            encrypt(&key, chunk).expect("encrypt of each block should succeed");
        }

        assert_ne!(
            buffer, original_buffer,
            "ciphertext must differ from plaintext"
        );

        for chunk in buffer.chunks_mut(key.key_size) {
            decrypt(&key, chunk).expect("decrypt of each block should succeed");
        }
        assert_eq!(buffer, original_buffer, "decrypt must restore original");
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

        let expected_plaintext = vec![0x78u8; key.key_size];
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
    fn encrypt_rejects_empty_data() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        let result = encrypt(&key, &mut []);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn decrypt_rejects_empty_data() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");
        let result = decrypt(&key, &mut []);
        assert!(matches!(result, Err(RsaError::InvalidKey)));
    }

    #[test]
    fn decrypt_crt_matches_naive() {
        let key = load_pem(RSA_PRIVATE_KEY_PEM).expect("valid PEM should load");

        // Test that CRT decryption produces the same result as the
        // naive c^d mod n path for several inputs.
        let test_vectors: &[&[u8]] = &[
            &[0x01u8; 128],
            &[0xFFu8; 128],
            &[0x78u8; 128],
            &[0x00u8; 128],
        ];

        for (i, &input) in test_vectors.iter().enumerate() {
            let mut crt_result = input.to_vec();
            decrypt(&key, &mut crt_result).expect("CRT decrypt should succeed");

            // Naive path: c^d mod n
            let ciphertext = BigUint::from_bytes_be(input);
            let expected_int = ciphertext.modpow(&key.private_exponent, &key.modulus);
            let mut expected = vec![0u8; key.key_size];
            write_bigint_be(&expected_int, &mut expected);

            assert_eq!(
                crt_result, expected,
                "CRT and naive decryption must match for vector {i}"
            );
        }
    }
}
