//! Seeded pseudo-random vectors for tests.

/// Returns `len` splitmix64 values mapped to `[-1, 1)`.
pub(crate) fn seeded(seed: u64, len: usize) -> Vec<f32> {
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            ((z >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
        })
        .collect()
}

/// Returns `len` approximately Gaussian values: the sum of four [`seeded`] streams.
pub(crate) fn seeded_normal(seed: u64, len: usize) -> Vec<f32> {
    let streams: Vec<Vec<f32>> = (0..4)
        .map(|k| seeded(seed.wrapping_mul(4).wrapping_add(k), len))
        .collect();
    (0..len)
        .map(|i| streams.iter().map(|s| s[i]).sum::<f32>() * 0.5)
        .collect()
}
