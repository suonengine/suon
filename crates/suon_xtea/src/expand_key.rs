use crate::{XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey, XTEARoundKeys};

/// Expands a 128-bit XTEA key into round keys for encryption/decryption.
///
/// The XTEA algorithm uses 32 rounds of encryption, and each round requires
/// two 32-bit keys derived from the original 128-bit key. This function
/// generates all the round keys in advance for efficient encryption/decryption.
///
/// # Arguments
/// * `key` - A reference to the 128-bit key represented as `[u32; 4]`.
///
/// # Returns
/// A `XTEARoundKeys` array containing `XTEA_NUM_ROUNDS * 2` 32-bit keys.
pub fn expand_key(key: &XTEAKey) -> XTEARoundKeys {
    // Create the output array for all round keys (two keys per round)
    let mut expanded = [0u32; XTEA_NUM_ROUNDS * 2];

    // Initialize the sum used in key scheduling
    let mut sum: u32 = 0;
    // Precompute the next sum for the first pair of keys
    let mut next_sum = sum.wrapping_add(XTEA_DELTA);

    // Loop over the expanded array two elements at a time (left/right word per round)
    for i in (0..expanded.len()).step_by(2) {
        // Compute the first key for this round: sum + key indexed by lower 2 bits of sum
        expanded[i] = sum.wrapping_add(key[(sum & 3) as usize]);
        // Compute the second key for this round: next_sum + key indexed by bits 11..12 of next_sum
        expanded[i + 1] = next_sum.wrapping_add(key[((next_sum >> 11) & 3) as usize]);

        // Update sums for the next iteration
        sum = next_sum;
        next_sum = next_sum.wrapping_add(XTEA_DELTA);
    }

    // Return the array containing all round keys
    expanded
}
