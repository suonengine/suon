use crate::{XTEA_DELTA, XTEA_NUM_ROUNDS, XTEAKey, XTEARoundKeys};

/// Expands a 128-bit XTEA key into a set of round keys for encryption/decryption.
///
/// XTEA uses 32 rounds, and each round requires two 32-bit subkeys derived from the original key.
/// This function precomputes all 64 subkeys for efficient encryption/decryption.
///
/// # Arguments
/// - `key`: A reference to the 128-bit key represented as `[u32; 4]`.
///
/// # Returns
/// An array of `XTEARoundKeys` containing `XTEA_NUM_ROUNDS * 2` 32-bit subkeys.
pub fn expand_key(key: &XTEAKey) -> XTEARoundKeys {
    // Prepare the array to hold 64 subkeys (2 per round for 32 rounds)
    let mut expanded = [0u32; XTEA_NUM_ROUNDS * 2];

    // Initialize sum for key schedule; starts at 0
    let mut sum: u32 = 0;
    // The next sum after adding delta
    let mut next_sum = sum.wrapping_add(XTEA_DELTA);

    // Loop over the array, filling two subkeys per iteration
    for i in (0..expanded.len()).step_by(2) {
        // Subkey for this round: sum + key indexed by lower 2 bits of sum
        expanded[i] = sum.wrapping_add(key[(sum & 3) as usize]);
        // Subkey for this round: next_sum + key indexed by bits 11..12 of next_sum
        expanded[i + 1] = next_sum.wrapping_add(key[((next_sum >> 11) & 3) as usize]);

        // Update sums for the next iteration
        sum = next_sum;
        next_sum = next_sum.wrapping_add(XTEA_DELTA);
    }

    expanded
}
