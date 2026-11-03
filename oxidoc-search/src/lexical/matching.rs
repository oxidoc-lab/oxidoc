use super::levenshtein::Levenshtein;

/// Find matching posting keys for a query token.
/// Returns (key, discount, is_fuzzy) tuples. is_fuzzy=true for Levenshtein matches.
pub(super) fn find_matching_postings<'a>(
    token: &str,
    keys: &[&'a String],
) -> Vec<(&'a str, f32, bool)> {
    let mut matches: Vec<(&'a str, f32, bool)> = Vec::new();

    let max_dist = max_edit_distance(token.len());
    let mut lev = Levenshtein::default();

    for key in keys {
        if key.as_str() == token {
            matches.push((key.as_str(), 1.0, false));
        } else if key.starts_with(token) {
            matches.push((key.as_str(), 0.9, false));
        } else if find_bytes(key, token).is_some() {
            let coverage = token.len() as f32 / key.len() as f32;
            let discount = 0.1 + coverage * 0.3;
            matches.push((key.as_str(), discount, false));
        } else if max_dist > 0 {
            let len_diff = key.len().abs_diff(token.len());
            if len_diff <= max_dist {
                if let Some(dist) = lev.within(token, key, max_dist)
                    && dist > 0
                {
                    let discount = match dist {
                        1 => 0.7,
                        2 => 0.4,
                        _ => 0.2,
                    };
                    matches.push((key.as_str(), discount, true));
                }
            } else if key.len() > token.len() {
                let prefix = &key[..floor_char_boundary(key, token.len())];
                if let Some(dist) = lev.within(token, prefix, max_dist)
                    && dist > 0
                {
                    let discount = match dist {
                        1 => 0.6,
                        2 => 0.3,
                        _ => 0.15,
                    };
                    matches.push((key.as_str(), discount, true));
                }
            }
        }
    }

    matches
}

/// Returns the byte offset of the first `needle` in `haystack`, as [`str::find`] does.
///
/// A byte match of valid UTF-8 always starts on a char boundary, so byte search gives
/// the same offset without building a searcher per call.
pub(super) fn find_bytes(haystack: &str, needle: &str) -> Option<usize> {
    find_bytes_from(haystack.as_bytes(), needle.as_bytes(), 0)
}

/// Returns the offset of the first `pat` in `hay` that starts at or after `from`.
///
/// An empty `pat` matches at `from` when `from` is inside `hay` or at its end.
pub(super) fn find_bytes_from(hay: &[u8], pat: &[u8], from: usize) -> Option<usize> {
    let Some((&first, rest)) = pat.split_first() else {
        return (from <= hay.len()).then_some(from);
    };
    let last_start = hay.len().checked_sub(pat.len())?;
    let mut from = from;
    while from <= last_start {
        let at = from + hay[from..=last_start].iter().position(|&b| b == first)?;
        if rest.iter().zip(&hay[at + 1..]).all(|(p, h)| p == h) {
            return Some(at);
        }
        from = at + 1;
    }
    None
}

/// Build a context snippet around a known offset.
///
/// `lower` is `text` lowercased. `offset` is a byte offset into `lower`.
pub(super) fn context_snippet_at(
    text: &str,
    lower: &str,
    offset: usize,
    matched_terms: &[&str],
    max_len: usize,
) -> String {
    // Find the matched term length at this offset
    let at = lower.get(offset..).unwrap_or("");
    let match_len = matched_terms
        .iter()
        .find(|t| at.starts_with(*t))
        .map_or(0, |t| t.len());

    let before = max_len / 3;
    let start = floor_char_boundary(text, offset.saturating_sub(before));
    let end = floor_char_boundary(text, (start + max_len).min(text.len()));
    let end = end.max(ceil_char_boundary(
        text,
        (offset + match_len + 20).min(text.len()),
    ));

    // Align to word boundaries
    let start = if start > 0 {
        text[start..]
            .find(' ')
            .map(|i| start + i + 1)
            .unwrap_or(start)
    } else {
        0
    };
    let end = if end < text.len() {
        text[..end].rfind(' ').unwrap_or(end)
    } else {
        text.len()
    };

    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str("...");
    }
    snippet.push_str(text[start..end].trim());
    if end < text.len() {
        snippet.push_str("...");
    }
    snippet
}

/// Round `idx` down to the nearest UTF-8 char boundary in `text`.
fn floor_char_boundary(text: &str, mut idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Round `idx` up to the nearest UTF-8 char boundary in `text`.
fn ceil_char_boundary(text: &str, mut idx: usize) -> usize {
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

/// Max edit distance based on term length.
/// Short terms (1-3 chars): 0 (no fuzzy)
/// Medium terms (4-6 chars): 1
/// Longer terms (7+): 2
pub(super) fn max_edit_distance(len: usize) -> usize {
    if len <= 3 {
        0
    } else if len <= 6 {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_snippet_multibyte_char_boundary() {
        // Regression: slicing at a byte index inside a multi-byte UTF-8 char
        // (here, an em dash) must not panic.
        let text = "API Reference — OpenAPI Integration Guide with extra trailing context words here so the window actually slides across the em dash boundary and triggers the failure case";
        let terms = ["api"];
        let s = context_snippet_at(text, &text.to_lowercase(), 0, &terms, 40);
        assert!(!s.is_empty());
    }

    #[test]
    fn find_bytes_matches_str_find() {
        let hays = [
            "tensor",
            "attention",
            "über-tensor",
            "",
            "aaab",
            "café",
            "a long haystack of more than sixteen bytes with tensor near its end",
        ];
        let needles = [
            "", "t", "ten", "tion", "sor", "ab", "é", "fé", "tensors", "x",
        ];
        for hay in hays {
            for needle in needles {
                assert_eq!(
                    find_bytes(hay, needle),
                    hay.find(needle),
                    "{hay} / {needle}"
                );
                for from in 0..=hay.len() {
                    let want = hay
                        .get(from..)
                        .and_then(|tail| tail.find(needle))
                        .map(|at| from + at);
                    if hay.is_char_boundary(from) {
                        let got = find_bytes_from(hay.as_bytes(), needle.as_bytes(), from);
                        assert_eq!(got, want, "{hay} / {needle} / {from}");
                    }
                }
            }
        }
    }

    #[test]
    fn test_max_edit_distance() {
        assert_eq!(max_edit_distance(2), 0); // short: no fuzzy
        assert_eq!(max_edit_distance(3), 0);
        assert_eq!(max_edit_distance(4), 1);
        assert_eq!(max_edit_distance(6), 1);
        assert_eq!(max_edit_distance(7), 2);
    }
}
