//! Symmetric per-vector int8 quantisation.

/// Largest stored magnitude. `-128` is never stored, so the range is symmetric.
pub const Q_MAX: i8 = 127;

/// Appends the int8 form of `vector` to `out` and returns its scale.
///
/// The scale is `max|x| / 127`. Each value is `round(x / scale)`, clamped to
/// `[-127, 127]`. Each element dequantises to `q * scale`, within `scale / 2` of `x`.
///
/// An all-zero vector gets scale 0 and zeros. So does a vector whose scale
/// underflows to 0. Returns `None` and appends nothing when `vector` holds NaN
/// or infinity.
pub fn quantize_into(vector: &[f32], out: &mut Vec<i8>) -> Option<f32> {
    let mut max = 0.0f32;
    for &x in vector {
        if !x.is_finite() {
            return None;
        }
        max = max.max(x.abs());
    }
    let scale = max / f32::from(Q_MAX);
    if scale == 0.0 {
        out.resize(out.len() + vector.len(), 0);
        return Some(0.0);
    }
    let limit = f32::from(Q_MAX);
    // The clamp bounds the value to [-127, 127], so the cast never saturates.
    out.extend(
        vector
            .iter()
            .map(|&x| (x / scale).round().clamp(-limit, limit) as i8),
    );
    Some(scale)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectors::seeded::seeded;

    fn quantize(vector: &[f32]) -> (f32, Vec<i8>) {
        let mut out = Vec::new();
        let scale = quantize_into(vector, &mut out).unwrap();
        (scale, out)
    }

    #[test]
    fn largest_magnitude_maps_to_127() {
        let (scale, q) = quantize(&[0.5, -1.0, 0.25]);
        assert_eq!(scale, 1.0 / 127.0);
        assert_eq!(q, vec![64, -127, 32]);
    }

    #[test]
    fn zero_vector_gets_zero_scale() {
        assert_eq!(quantize(&[0.0, -0.0, 0.0]), (0.0, vec![0, 0, 0]));
        assert_eq!(quantize(&[]), (0.0, vec![]));
    }

    #[test]
    fn underflowing_scale_is_zero_vector() {
        let tiny = f32::from_bits(1);
        assert_eq!(quantize(&[tiny, -tiny]), (0.0, vec![0, 0]));
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let mut out = vec![7];
        assert_eq!(quantize_into(&[1.0, f32::NAN], &mut out), None);
        assert_eq!(quantize_into(&[f32::INFINITY], &mut out), None);
        assert_eq!(quantize_into(&[f32::NEG_INFINITY, 0.0], &mut out), None);
        assert_eq!(out, vec![7]);
    }

    #[test]
    fn appends_after_existing_values() {
        let mut out = vec![1, 2];
        quantize_into(&[2.0, -2.0], &mut out).unwrap();
        assert_eq!(out, vec![1, 2, 127, -127]);
    }

    #[test]
    fn error_is_at_most_half_a_step() {
        for seed in 0..200 {
            let vector: Vec<f32> = seeded(seed, 384)
                .into_iter()
                .map(|x| x * (seed as f32 + 1.0) * 0.01)
                .collect();
            let (scale, q) = quantize(&vector);
            assert!(scale > 0.0);
            // f32 rounding of `x / scale` adds at most a few ulps of 127.
            let bound = f64::from(scale) * (0.5 + 1e-5);
            for (&x, &v) in vector.iter().zip(&q) {
                assert!(v != i8::MIN);
                let err = (f64::from(x) - f64::from(v) * f64::from(scale)).abs();
                assert!(err <= bound, "seed {seed}: |{x} - {v} * {scale}| = {err}");
            }
        }
    }
}
