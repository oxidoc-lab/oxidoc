//! Bounded Levenshtein edit distance with reusable buffers.

/// Reusable buffers for [`Levenshtein::within`].
///
/// One value serves many calls, so the hot path does not allocate per word.
#[derive(Default)]
pub(super) struct Levenshtein {
    /// Chars of the first string, for non-ASCII input.
    a: Vec<char>,
    /// Chars of the second string, for non-ASCII input.
    b: Vec<char>,
    /// Previous row of the distance table.
    prev: Vec<usize>,
    /// Current row of the distance table.
    curr: Vec<usize>,
}

impl Levenshtein {
    /// Returns the char edit distance of `a` and `b` when it is at most `max`.
    ///
    /// Returns `None` when the distance is above `max`. ASCII input compares bytes.
    /// Other input compares chars.
    pub(super) fn within(&mut self, a: &str, b: &str, max: usize) -> Option<usize> {
        let Self {
            a: a_chars,
            b: b_chars,
            prev,
            curr,
        } = self;
        if a.is_ascii() && b.is_ascii() {
            return bounded(a.as_bytes(), b.as_bytes(), max, prev, curr);
        }
        a_chars.clear();
        a_chars.extend(a.chars());
        b_chars.clear();
        b_chars.extend(b.chars());
        bounded(a_chars, b_chars, max, prev, curr)
    }
}

/// Banded two-row edit distance of `a` and `b`, or `None` when it exceeds `max`.
///
/// A cell more than `max` off the diagonal costs more than `max`, so each row computes
/// only the band `|i - j| <= max`. Cells hold at most `max + 1`. Saturating keeps every
/// distance up to `max` exact. The minimum of a row never decreases in later rows, so
/// a row above `max` ends the search.
fn bounded<T: PartialEq>(
    a: &[T],
    b: &[T],
    max: usize,
    prev: &mut Vec<usize>,
    curr: &mut Vec<usize>,
) -> Option<usize> {
    if a.len().abs_diff(b.len()) > max {
        return None;
    }
    let n = b.len();
    let over = max + 1;
    prev.clear();
    prev.extend((0..=n).map(|j| j.min(over)));
    curr.clear();
    curr.resize(n + 1, over);
    for (i, ca) in a.iter().enumerate() {
        let row = i + 1;
        let lo = row.saturating_sub(max).max(1);
        let hi = (row + max).min(n);
        // The cell left of the band: column 0, or a cell off the band.
        curr[lo - 1] = if lo == 1 { row.min(over) } else { over };
        let mut row_min = curr[lo - 1];
        for j in lo..=hi {
            let cost = usize::from(*ca != b[j - 1]);
            let cell = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            let cell = cell.min(over);
            curr[j] = cell;
            row_min = row_min.min(cell);
        }
        // The next row reads this cell as the one right of its band.
        if hi < n {
            curr[hi + 1] = over;
        }
        if row_min > max {
            return None;
        }
        std::mem::swap(prev, curr);
    }
    let dist = prev[n];
    (dist <= max).then_some(dist)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unbounded reference distance over chars.
    fn reference(a: &str, b: &str) -> usize {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let mut prev: Vec<usize> = (0..=b.len()).collect();
        for (i, ca) in a.iter().enumerate() {
            let mut curr = vec![i + 1; b.len() + 1];
            for (j, cb) in b.iter().enumerate() {
                let cost = usize::from(ca != cb);
                curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
            }
            prev = curr;
        }
        prev[b.len()]
    }

    #[test]
    fn within_table() {
        let mut lev = Levenshtein::default();
        let cases = [
            ("hello", "hello", 2, Some(0)),
            ("glock", "block", 2, Some(1)),
            ("blck", "block", 2, Some(1)),
            ("blocx", "block", 1, Some(1)),
            ("glok", "block", 2, Some(2)),
            ("glok", "block", 1, None),
            ("", "ab", 2, Some(2)),
            ("abc", "", 2, None),
            ("café", "cafe", 1, Some(1)),
            ("naïve", "naive", 0, None),
            ("tensor", "kernel", 2, None),
        ];
        for (a, b, max, want) in cases {
            assert_eq!(lev.within(a, b, max), want, "{a} / {b} / {max}");
        }
    }

    #[test]
    fn within_matches_reference() {
        let words = [
            "tensor",
            "tensro",
            "tens",
            "gradient",
            "gradeint",
            "transformer",
            "trasformer",
            "kernal",
            "kernel",
            "über",
            "uber",
            "x",
            "",
            "ab",
            "ba",
            "abcdef",
            "badcfe",
            "kitten",
            "sitting",
            "flaw",
            "lawn",
            "tensortensor",
            "tensrotensor",
        ];
        let mut lev = Levenshtein::default();
        for a in words {
            for b in words {
                let dist = reference(a, b);
                for max in 0..4 {
                    let want = (dist <= max).then_some(dist);
                    assert_eq!(lev.within(a, b, max), want, "{a} / {b} / {max}");
                }
            }
        }
    }
}
