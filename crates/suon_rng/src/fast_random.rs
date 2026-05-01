use bevy::prelude::*;
use bevy_rand::prelude::WyRand;
use core::ops::{Deref, DerefMut};
use rand_core::Rng;

use crate::{distribution::RngDistributionExt, seed::seed_rng, settings::RngSettings};

/// Default RNG algorithm for hot paths that only need lightweight randomness.
pub type FastRandomAlgorithm = WyRand;

/// Fast deterministic RNG resource backed by WyRand.
#[derive(Debug, Clone, Resource)]
pub struct FastRandom(FastRandomAlgorithm);

impl Default for FastRandom {
    fn default() -> Self {
        Self::seed_from_u64(RngSettings::default().fast_seed())
    }
}

impl FastRandom {
    /// Creates a deterministic fast RNG from a compact numeric seed.
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

impl Deref for FastRandom {
    type Target = FastRandomAlgorithm;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FastRandom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_deterministic_fast_random_sequence_from_seed() {
        let mut first = FastRandom::seed_from_u64(42);
        let mut second = FastRandom::seed_from_u64(42);

        assert_eq!(first.next_u64(), second.next_u64());
        assert_eq!(first.next_u64(), second.next_u64());
    }

    #[test]
    fn should_generate_deterministic_u32_sequence_from_seed() {
        let mut first = FastRandom::seed_from_u64(42);
        let mut second = FastRandom::seed_from_u64(42);

        assert_eq!(first.next_u32(), second.next_u32());
        assert_eq!(first.next_u32(), second.next_u32());
    }

    #[test]
    fn should_generate_values_inside_inclusive_range() {
        let mut random = FastRandom::seed_from_u64(7);

        for _ in 0..128 {
            let value = random.range_u32(10, 20);
            assert!((10..=20).contains(&value));
        }
    }

    #[test]
    fn should_respect_certain_and_impossible_chances() {
        let mut random = FastRandom::seed_from_u64(99);
        assert!(!random.chance(0, 100));
        assert!(random.chance(100, 100));
    }

    #[test]
    fn should_fill_bytes_deterministically_from_seed() {
        let mut first = FastRandom::seed_from_u64(55);
        let mut second = FastRandom::seed_from_u64(55);

        let mut first_bytes = [0; 16];
        let mut second_bytes = [0; 16];

        first.fill_bytes(&mut first_bytes);
        second.fill_bytes(&mut second_bytes);

        assert_eq!(first_bytes, second_bytes);
        assert_ne!(first_bytes, [0; 16]);
    }
}
