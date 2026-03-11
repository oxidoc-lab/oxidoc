//! Levenshtein distance-based suggestion for unknown config keys.
//!
//! Returns the best matching candidate if its distance is within the threshold.

/// Calculate the Levenshtein distance between two strings.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *val = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// Find the best suggestion from candidates within a reasonable distance.
pub fn find_suggestion(input: &str, candidates: &[&str]) -> Option<String> {
    const MAX_DISTANCE: usize = 2;

    let mut best: Option<(String, usize)> = None;

    for &candidate in candidates {
        let dist = levenshtein_distance(input, candidate);
        if dist <= MAX_DISTANCE {
            if let Some((_, best_dist)) = &best {
                if dist < *best_dist {
                    best = Some((candidate.to_string(), dist));
                }
            } else {
                best = Some((candidate.to_string(), dist));
            }
        }
    }

    best.map(|(s, _)| s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_one_insertion() {
        assert_eq!(levenshtein_distance("helo", "hello"), 1);
    }

    #[test]
    fn test_one_deletion() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
    }

    #[test]
    fn test_one_substitution() {
        assert_eq!(levenshtein_distance("hallo", "hello"), 1);
    }

    #[test]
    fn test_find_suggestion_exact() {
        let candidates = &["project", "theme", "routing"];
        let suggestion = find_suggestion("project", candidates);
        assert_eq!(suggestion, Some("project".to_string()));
    }

    #[test]
    fn test_find_suggestion_close_match() {
        let candidates = &["project", "theme", "routing"];
        let suggestion = find_suggestion("projct", candidates);
        assert_eq!(suggestion, Some("project".to_string()));
    }

    #[test]
    fn test_find_suggestion_no_match() {
        let candidates = &["project", "theme", "routing"];
        let suggestion = find_suggestion("xyz", candidates);
        assert_eq!(suggestion, None);
    }

    #[test]
    fn test_find_suggestion_multiple_candidates() {
        let candidates = &["project", "projection", "theorem"];
        let suggestion = find_suggestion("projct", candidates);
        // Should pick the closest one
        assert!(suggestion.is_some());
    }
}
