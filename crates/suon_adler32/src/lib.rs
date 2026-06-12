//! Adler-32 checksum algorithm as used in the zlib / RS³ protocol.
//!
//! # Algorithm overview
//!
//! Adler-32 is a checksum algorithm that is cheaper than CRC-32 while
//! providing comparable error detection for small to medium payloads.
//! It treats the input as a stream of bytes and maintains two running
//! 16-bit accumulators:
//!
//! - **A** — sum of all input bytes (modulo 65521, the largest prime
//!   smaller than 2¹⁶).
//! - **B** — sum of every **A** value encountered so far (again modulo
//!   65521).
//!
//! The final checksum is `(B << 16) | A`.
//!
//! ```text
//! A = 1 + Σ input[i]   (mod 65521)
//! B = Σ A[i]            (mod 65521)
//! checksum = (B << 16) | A
//! ```
//!
//! # Example
//!
//! ```
//! let checksum = suon_adler32::generate(b"Wikipedia");
//! assert_eq!(checksum, 0x11E60398);
//! ```

#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

/// Computes the Adler-32 checksum of `data`.
///
/// Returns a 32-bit unsigned integer whose upper 16 bits are **B** and
/// lower 16 bits are **A** (see [module-level documentation](self) for
/// the definition of A and B).
///
/// # Example
///
/// ```
/// let sum = suon_adler32::generate(b"hello");
/// assert_eq!(sum, 103547413);
/// ```
pub fn generate(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_wikipedia_vector() {
        assert_eq!(generate(b"Wikipedia"), 0x11E60398);
    }

    #[test]
    fn empty_input() {
        assert_eq!(generate(b""), 1);
    }

    #[test]
    fn single_byte() {
        assert_eq!(generate(b"a"), 0x00620062);
    }

    #[test]
    fn deterministic() {
        let data = b"the quick brown fox";
        assert_eq!(generate(data), generate(data));
    }

    #[test]
    fn different_inputs_differ() {
        assert_ne!(generate(b"abc"), generate(b"xyz"));
    }

    #[test]
    fn order_matters() {
        assert_ne!(generate(b"ab"), generate(b"abc"));
    }

    #[test]
    fn long_input_correctness() {
        let data = vec![0xABu8; 4096];
        let result = generate(&data);
        // Just verify it runs without overflow and returns non-zero
        assert_ne!(result, 0);
        assert_eq!(generate(&data), generate(&data));
    }

    #[test]
    fn large_input_no_panic() {
        let data = vec![0xFFu8; 1_000_000];
        let result = generate(&data);
        assert!(result > 0);
    }

    #[test]
    fn all_zeros() {
        let data = vec![0u8; 100];
        let result = generate(&data);
        // Known value for 100 zero bytes:
        // A = 1 + 0*100 = 1 (mod 65521)
        // B = 1*100 = 100 (mod 65521) = 100
        assert_eq!(result, (100 << 16) | 1);
    }

    #[test]
    fn all_ones() {
        let data = vec![0xFFu8; 10];
        let result = generate(&data);
        assert!(result > 0);
    }

    #[test]
    fn maximal_values_mod_prime() {
        let data = vec![0xFFu8; 256];
        let result = generate(&data);
        assert!(result > 0);
    }
}
