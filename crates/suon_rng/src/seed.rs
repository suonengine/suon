use rand_core::SeedableRng;

pub(crate) fn seed_rng<R>(mut state: u64) -> R
where
    R: SeedableRng,
    R::Seed: Default + AsMut<[u8]>,
{
    let mut seed = R::Seed::default();

    for chunk in seed.as_mut().chunks_mut(size_of::<u64>()) {
        let bytes = splitmix64(&mut state).to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }

    R::from_seed(seed)
}

pub(crate) fn splitmix64_value(mut state: u64) -> u64 {
    splitmix64(&mut state)
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);

    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
