use rust_stemmers::{Algorithm, Stemmer};
use unicode_normalization::UnicodeNormalization;

/// Common English stop words to filter out of search indices.
const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into", "is", "it",
    "no", "not", "of", "on", "or", "such", "that", "the", "their", "then", "there", "these",
    "they", "this", "to", "was", "will", "with", "you", "your", "can", "had", "have", "has", "do",
    "does", "did", "from", "how", "its", "may", "more", "most", "must", "than", "what", "when",
    "which", "who",
];

/// Check if a word is a stop word.
pub fn is_stop_word(word: &str) -> bool {
    STOP_WORDS.binary_search(&word).is_ok()
}

/// Unicode NFD normalize and strip combining marks (diacritics).
pub fn normalize_unicode(s: &str) -> String {
    s.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .collect()
}

/// Stem a word using the Snowball English stemmer.
pub fn stem(word: &str) -> String {
    let stemmer = Stemmer::create(Algorithm::English);
    stemmer.stem(word).into_owned()
}

/// Split a camelCase or PascalCase string into its component words.
/// e.g. "CodeBlock" -> ["Code", "Block"], "myFunc" -> ["my", "Func"]
pub fn split_camel_case(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let bytes = s.as_bytes();
    let mut start = 0;
    for i in 1..bytes.len() {
        let curr_upper = bytes[i].is_ascii_uppercase();
        let prev_upper = bytes[i - 1].is_ascii_uppercase();
        // Split at lowercase->uppercase boundary (e.g. "code|Block")
        // Split at uppercase->uppercase->lowercase boundary (e.g. "XML|Parser")
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

/// Tokenize text into normalized, stemmed terms.
///
/// Pipeline:
/// 1. Split on whitespace
/// 2. Strip non-alphanumeric (keep `-`, `_`)
/// 3. Skip empty / single-char tokens
/// 4. Unicode NFD normalize + strip combining marks
/// 5. Lowercase
/// 6. Split camelCase (emit compound + parts)
/// 7. Filter stop words
/// 8. Stem via Snowball English stemmer
/// 9. Return Vec<String>
pub fn tokenize(text: &str) -> Vec<String> {
    let mut result = Vec::new();

    for (stem, _raw) in tokenize_with_raw(text) {
        result.push(stem);
    }
    result
}

/// Like [`tokenize`] but returns `(stemmed, raw_lowercased)` pairs.
///
/// The raw form is useful for query-time matching: a partial-word query like
/// "analy" stems to "anali" (Snowball's trailing y→i rule) and matches no
/// indexed term, but "analy" as a raw prefix still matches stemmed postings
/// like "analyt". Callers can try both forms.
///
/// Applies stop-word filtering (matches the index pipeline). Use
/// [`tokenize_query`] at search time to keep partial tokens like `"an"`
/// (prefix of "analytics") that would otherwise be dropped as stop words.
pub fn tokenize_with_raw(text: &str) -> Vec<(String, String)> {
    tokenize_with_raw_inner(text, true)
}

/// Query-time variant of [`tokenize_with_raw`] that does NOT drop stop words.
///
/// The index skips stop words for compactness, but a short query like `"an"`
/// is usually a partial word (prefix of "analytics"), not a search for the
/// article "an". Keep it so it can prefix-match stemmed postings.
pub fn tokenize_query(text: &str) -> Vec<(String, String)> {
    tokenize_with_raw_inner(text, false)
}

fn tokenize_with_raw_inner(text: &str, filter_stop_words: bool) -> Vec<(String, String)> {
    let stemmer = Stemmer::create(Algorithm::English);
    let mut result = Vec::new();

    for word in text.split_whitespace() {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if cleaned.is_empty() {
            continue;
        }

        let normalized = normalize_unicode(&cleaned);
        let lower = normalized.to_lowercase();

        if lower.len() <= 1 {
            continue;
        }

        let keep = |w: &str| -> bool { !filter_stop_words || !is_stop_word(w) };

        let parts = split_camel_case(&cleaned);
        if parts.len() > 1 {
            if keep(&lower) {
                result.push((stemmer.stem(&lower).into_owned(), lower.clone()));
            }
            for part in parts {
                let normalized_part = normalize_unicode(part);
                let p = normalized_part.to_lowercase();
                if p.len() > 1 && keep(&p) {
                    result.push((stemmer.stem(&p).into_owned(), p));
                }
            }
        } else if keep(&lower) {
            result.push((stemmer.stem(&lower).into_owned(), lower));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("Hello World");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_stemming() {
        let tokens = tokenize("running jumps connected");
        assert!(tokens.contains(&"run".to_string()));
        assert!(tokens.contains(&"jump".to_string()));
        assert!(tokens.contains(&"connect".to_string()));
    }

    #[test]
    fn test_tokenize_stop_words() {
        let tokens = tokenize("the quick and brown fox");
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"and".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
        assert!(tokens.contains(&"brown".to_string()));
        assert!(tokens.contains(&"fox".to_string()));
    }

    #[test]
    fn test_tokenize_camel_case() {
        let tokens = tokenize("CodeBlock");
        // compound "codeblock" stemmed + parts "code" and "block"
        assert!(tokens.contains(&"codeblock".to_string()));
        assert!(tokens.contains(&"code".to_string()));
        assert!(tokens.contains(&"block".to_string()));
    }

    #[test]
    fn test_tokenize_preserves_hyphens() {
        let tokens = tokenize("rust-lang is great");
        assert!(tokens.contains(&"rust-lang".to_string()));
    }

    #[test]
    fn test_tokenize_filters_short_tokens() {
        let tokens = tokenize("a b cd efgh");
        assert!(!tokens.iter().any(|t| t == "a"));
        assert!(!tokens.iter().any(|t| t == "b"));
    }

    #[test]
    fn test_tokenize_unicode_normalization() {
        let tokens = tokenize("caf\u{00e9} na\u{00ef}ve");
        assert!(tokens.contains(&"cafe".to_string()));
        assert!(tokens.contains(&"naiv".to_string()));
    }

    #[test]
    fn test_stem() {
        assert_eq!(stem("running"), "run");
        assert_eq!(stem("documentation"), "document");
    }

    #[test]
    fn test_is_stop_word() {
        assert!(is_stop_word("the"));
        assert!(is_stop_word("and"));
        assert!(!is_stop_word("rust"));
    }

    #[test]
    fn test_split_camel_case() {
        assert_eq!(split_camel_case("CodeBlock"), vec!["Code", "Block"]);
        assert_eq!(split_camel_case("myFunc"), vec!["my", "Func"]);
        assert_eq!(split_camel_case("XMLParser"), vec!["XML", "Parser"]);
        assert_eq!(split_camel_case("hello"), vec!["hello"]);
    }
}
