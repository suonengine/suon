use bevy::prelude::*;
use bevy_rand::prelude::ChaCha8Rng;
use core::ops::{Deref, DerefMut};
use rand_core::Rng;

use crate::{distribution::RngDistributionExt, seed::seed_rng, settings::RngSettings};

/// Default RNG algorithm for gameplay rules that prefer reproducibility.
pub type RandomAlgorithm = ChaCha8Rng;

/// Deterministic RNG resource backed by ChaCha8.
#[derive(Debug, Clone, Resource)]
pub struct Random(RandomAlgorithm);

impl Default for Random {
    fn default() -> Self {
        Self::seed_from_u64(RngSettings::default().seed())
    }
}

impl Random {
    /// Creates a deterministic RNG from a compact numeric seed.
    pub fn seed_from_u64(seed: u64) -> Self {
        Self(seed_rng(seed))
    }

    /// Returns the next `u32` from the underlying RNG.
    pub fn next_u32(&mut self) -> u32 {
        self.0.next_u32()
    }

    /// Returns the next `u64` from the underlying RNG.
    pub fn next_u64(&mut self) -> u64 {
        self.0.next_u64()
    }

    /// Fills `dest` with random bytes.
    pub fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.fill_bytes(dest);
    }

    /// Returns a random `u32` in the inclusive range `min..=max`.
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        self.0.range_u32(min, max)
    }

    /// Returns true with `numerator / denominator` probability.
    pub fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
        self.0.chance(numerator, denominator)
    }
}

impl Deref for Random {
    type Target = RandomAlgorithm;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Random {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_deterministic_random_sequence_from_seed() {
        let mut first = Random::seed_from_u64(42);
        let mut second = Random::seed_from_u64(42);

        assert_eq!(first.next_u64(), second.next_u64());
        assert_eq!(first.next_u64(), second.next_u64());
    }

    #[test]
    fn should_generate_deterministic_u32_sequence_from_seed() {
        let mut first = Random::seed_from_u64(42);
        let mut second = Random::seed_from_u64(42);

        assert_eq!(first.next_u32(), second.next_u32());
        assert_eq!(first.next_u32(), second.next_u32());
    }

    #[test]
    fn should_fill_bytes_deterministically_from_seed() {
        let mut first = Random::seed_from_u64(55);
        let mut second = Random::seed_from_u64(55);
        let mut first_bytes = [0; 16];
        let mut second_bytes = [0; 16];

        first.fill_bytes(&mut first_bytes);
        second.fill_bytes(&mut second_bytes);

        assert_eq!(first_bytes, second_bytes);
        assert_ne!(first_bytes, [0; 16]);
    }

    #[test]
    fn should_generate_values_inside_inclusive_range() {
        let mut random = Random::seed_from_u64(7);

        for _ in 0..128 {
            let value = random.range_u32(10, 20);

            assert!((10..=20).contains(&value));
        }
    }

    #[test]
    fn range_should_return_the_bound_when_min_equals_max() {
        let mut random = Random::seed_from_u64(7);

        assert_eq!(random.range_u32(13, 13), 13);
    }

    #[test]
    #[should_panic(expected = "min must be less than or equal to max")]
    fn range_should_reject_inverted_bounds() {
        let mut random = Random::seed_from_u64(7);

        random.range_u32(20, 10);
    }

    #[test]
    #[should_panic(expected = "denominator must be greater than zero")]
    fn chance_should_reject_zero_denominator() {
        let mut random = Random::seed_from_u64(7);

        random.chance(1, 0);
    }

    #[test]
    #[should_panic(expected = "numerator must be less than or equal to denominator")]
    fn chance_should_reject_numerator_above_denominator() {
        let mut random = Random::seed_from_u64(7);

        random.chance(2, 1);
    }
}
