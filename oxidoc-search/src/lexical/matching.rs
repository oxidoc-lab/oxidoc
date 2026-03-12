/// Find matching posting keys for a query token.
/// Returns (key, discount, is_fuzzy) tuples. is_fuzzy=true for Levenshtein matches.
pub(super) fn find_matching_postings<'a>(
    token: &str,
    keys: &[&'a String],
) -> Vec<(&'a str, f32, bool)> {
    let mut matches: Vec<(&'a str, f32, bool)> = Vec::new();

    let max_dist = max_edit_distance(token.len());

    for key in keys {
        if key.as_str() == token {
            matches.push((key.as_str(), 1.0, false));
        } else if key.starts_with(token) {
            matches.push((key.as_str(), 0.9, false));
        } else if key.contains(token) {
            let coverage = token.len() as f32 / key.len() as f32;
            let discount = 0.1 + coverage * 0.3;
            matches.push((key.as_str(), discount, false));
        } else if max_dist > 0 {
            let len_diff = (key.len() as isize - token.len() as isize).unsigned_abs();
            if len_diff <= max_dist {
                let dist = levenshtein(token, key);
                if dist > 0 && dist <= max_dist {
                    let discount = match dist {
                        1 => 0.7,
                        2 => 0.4,
                        _ => 0.2,
                    };
                    matches.push((key.as_str(), discount, true));
                }
            } else if key.len() > token.len() {
                let prefix = &key[..token.len()];
                let dist = levenshtein(token, prefix);
                if dist > 0 && dist <= max_dist {
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

/// Find ALL character offsets where any matched term appears in the text.
pub(super) fn find_all_match_offsets(text: &str, matched_terms: &[String]) -> Vec<usize> {
    let lower = text.to_lowercase();
    let mut offsets = Vec::new();
    for term in matched_terms {
        let mut start = 0;
        while let Some(idx) = lower[start..].find(term.as_str()) {
            offsets.push(start + idx);
            start += idx + term.len();
        }
    }
    offsets.sort();
    offsets.dedup();
    offsets
}

/// Build a context snippet around a known offset.
pub(super) fn context_snippet_at(
    text: &str,
    offset: usize,
    matched_terms: &[String],
    max_len: usize,
) -> String {
    // Find the matched term length at this offset
    let lower = text.to_lowercase();
    let match_len = matched_terms
        .iter()
        .find(|t| lower[offset..].starts_with(t.as_str()))
        .map(|t| t.len())
        .unwrap_or(0);

    let before = max_len / 3;
    let start = offset.saturating_sub(before);
    let end = (start + max_len).min(text.len());
    let end = end.max((offset + match_len + 20).min(text.len()));

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

/// Compute Levenshtein edit distance between two strings.
pub(super) fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // Use two rows instead of full matrix for space efficiency
    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];

    for (j, slot) in prev.iter_mut().enumerate().take(n + 1) {
        *slot = j;
    }

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

pub(super) fn tokenize(text: &str) -> Vec<String> {
    let mut result = Vec::new();
    for word in text.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }
        let lower = cleaned.to_lowercase();
        if lower.len() <= 1 {
            continue;
        }
        // Split camelCase/PascalCase into sub-words
        let parts = split_camel_case(&cleaned);
        if parts.len() > 1 {
            result.push(lower);
            for part in parts {
                let p = part.to_lowercase();
                if p.len() > 1 {
                    result.push(p);
                }
            }
        } else {
            result.push(lower);
        }
    }
    result
}

/// Split a camelCase or PascalCase string into its component words.
pub(super) fn split_camel_case(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = s.as_bytes();
    let mut start = 0;
    for i in 1..bytes.len() {
        let curr_upper = bytes[i].is_ascii_uppercase();
        let prev_upper = bytes[i - 1].is_ascii_uppercase();
        if curr_upper && (!prev_upper || (i + 1 < bytes.len() && bytes[i + 1].is_ascii_lowercase()))
        {
            parts.push(&s[start..i]);
            start = i;
        }
    }
    if start < s.len() {
        parts.push(&s[start..]);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello World Rust!");
        assert_eq!(tokens, vec!["hello", "world", "rust"]);
    }

    #[test]
    fn test_tokenize_with_symbols() {
        let tokens = tokenize("rust-lang_2024");
        assert_eq!(tokens, vec!["rust-lang_2024"]);
    }

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein("glock", "block"), 1);
        assert_eq!(levenshtein("blck", "block"), 1);
        assert_eq!(levenshtein("blocx", "block"), 1);
    }

    #[test]
    fn test_levenshtein_two_edits() {
        assert_eq!(levenshtein("glok", "block"), 2);
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
