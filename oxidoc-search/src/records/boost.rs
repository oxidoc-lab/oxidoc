//! Clamping of per-document boosts.

/// Largest boost the runtime applies.
///
/// The widest ratio between two clamped boosts is `MAX_BOOST / MIN_BOOST` = 2.25.
/// An exact single-term name match earns a 3x heading match over a match elsewhere,
/// so a boost alone never lifts a non-match above an exact name match.
pub const MAX_BOOST: f32 = 1.5;

/// Smallest boost the runtime applies.
pub const MIN_BOOST: f32 = 1.0 / MAX_BOOST;

/// Returns `boost` clamped to `MIN_BOOST..=MAX_BOOST`. A non-finite boost gives 1.0.
pub fn effective_boost(boost: f32) -> f32 {
    if boost.is_finite() {
        boost.clamp(MIN_BOOST, MAX_BOOST)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_boost_table() {
        let cases = [
            (1.0, 1.0),
            (1.2, 1.2),
            (10.0, MAX_BOOST),
            (0.0, MIN_BOOST),
            (-3.0, MIN_BOOST),
            (f32::NAN, 1.0),
            (f32::INFINITY, 1.0),
        ];
        for (input, want) in cases {
            assert_eq!(effective_boost(input), want, "boost {input}");
        }
    }
}
