use crate::error::SearchResult;
use crate::index::deserialize_lexical_index;
use crate::types::{LexicalIndex, SearchQuery, SearchResult as DocResult, SearchSource};
use std::collections::HashMap;

pub struct LexicalSearcher {
    index: LexicalIndex,
}

impl LexicalSearcher {
    pub fn from_bytes(data: &[u8]) -> SearchResult<Self> {
        let index = deserialize_lexical_index(data)?;
        Ok(Self { index })
    }

    pub fn new(index: LexicalIndex) -> Self {
        Self { index }
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<DocResult> {
        let text = query.text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        let tokens = tokenize(text);
        if tokens.is_empty() {
            return Vec::new();
        }

        // Find which posting keys match each query token, and how
        let posting_keys: Vec<&String> = self.index.postings.keys().collect();
        let mut token_matches: Vec<Vec<(&str, f32, bool)>> = Vec::new();
        for token in &tokens {
            token_matches.push(find_matching_postings(token, &posting_keys));
        }

        // Find which documents have any matches
        let mut doc_ids: Vec<u32> = Vec::new();
        for matches in &token_matches {
            for (key, _, _) in matches {
                if let Some(postings) = self.index.postings.get(*key) {
                    for posting in postings {
                        if !doc_ids.contains(&posting.doc_id) {
                            doc_ids.push(posting.doc_id);
                        }
                    }
                }
            }
        }

        // Collect fuzzy keys per doc (for highlight_terms)
        let mut doc_fuzzy_keys: HashMap<u32, Vec<String>> = HashMap::new();
        for matches in &token_matches {
            for (key, _, is_fuzzy) in matches {
                if *is_fuzzy && let Some(postings) = self.index.postings.get(*key) {
                    for posting in postings {
                        doc_fuzzy_keys
                            .entry(posting.doc_id)
                            .or_default()
                            .push(key.to_string());
                    }
                }
            }
        }

        // Collect all matched posting keys per doc (for text offset finding)
        let mut doc_matched_keys: HashMap<u32, Vec<String>> = HashMap::new();
        for matches in &token_matches {
            for (key, _, _) in matches {
                if let Some(postings) = self.index.postings.get(*key) {
                    for posting in postings {
                        doc_matched_keys
                            .entry(posting.doc_id)
                            .or_default()
                            .push(key.to_string());
                    }
                }
            }
        }

        let num_tokens = tokens.len();
        let mut all_results: Vec<DocResult> = Vec::new();

        for doc_id in &doc_ids {
            let doc = match self.index.documents.iter().find(|d| d.id == *doc_id) {
                Some(d) => d,
                None => continue,
            };

            let mut terms = doc_matched_keys.remove(doc_id).unwrap_or_default();
            terms.sort();
            terms.dedup();

            let mut highlight_terms = doc_fuzzy_keys.remove(doc_id).unwrap_or_default();
            highlight_terms.sort();
            highlight_terms.dedup();

            if doc.text.is_empty() {
                all_results.push(DocResult {
                    title: doc.title.clone(),
                    path: doc.path.clone(),
                    snippet: doc.snippet.clone(),
                    score: 0.1,
                    source: SearchSource::Lexical,
                    breadcrumb: vec![],
                    anchor: String::new(),
                    highlight_terms,
                });
                continue;
            }

            // Find all match offsets, group by section
            let offsets = find_all_match_offsets(&doc.text, &terms);
            if offsets.is_empty() {
                continue;
            }

            let mut seen_anchors: Vec<String> = Vec::new();
            for offset in &offsets {
                let (breadcrumb, anchor) =
                    resolve_heading_breadcrumb(&doc.title, &doc.headings, Some(*offset));
                if seen_anchors.contains(&anchor) {
                    continue;
                }
                seen_anchors.push(anchor.clone());

                // Score this section directly, with heading boost
                let section_text = get_section_text(&doc.text, &doc.headings, *offset);
                let heading_title = if breadcrumb.len() > 1 {
                    breadcrumb.last().map(|s| s.as_str()).unwrap_or("")
                } else {
                    &doc.title
                };
                let score = score_section(section_text, heading_title, &tokens, num_tokens);

                if score <= 0.0 {
                    continue;
                }

                let snippet = context_snippet_at(&doc.text, *offset, &terms, 160);
                let path = if anchor.is_empty() {
                    doc.path.clone()
                } else {
                    format!("{}#{}", doc.path, anchor)
                };

                all_results.push(DocResult {
                    title: doc.title.clone(),
                    path,
                    snippet,
                    score,
                    source: SearchSource::Lexical,
                    breadcrumb,
                    anchor,
                    highlight_terms: highlight_terms.clone(),
                });
            }
        }

        all_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_results.truncate(query.max_results);
        all_results
    }
}

/// Score a section based on how query tokens match within it.
/// For each token, find the best matching word. Score = 1.0 - (match_position / word_length).
/// Earlier match in word = higher score. Exact match = 1.0.
/// Multi-token: all tokens matched → 3x bonus.
fn score_section(
    section_text: &str,
    heading_title: &str,
    tokens: &[String],
    num_tokens: usize,
) -> f32 {
    let lower = section_text.to_lowercase();
    let heading_lower = heading_title.to_lowercase();
    let heading_words: Vec<&str> = heading_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mut total = 0.0_f32;
    let mut matched_count = 0usize;

    for tk in tokens {
        let mut best = 0.0_f32;
        let max_dist = max_edit_distance(tk.len());
        for word in &words {
            // Literal substring match — scored by position and coverage
            if let Some(pos) = word.find(tk.as_str()) {
                let wlen = word.len().max(1) as f32;
                let position_score = 1.0 - (pos as f32 / wlen);
                let coverage_score = tk.len() as f32 / wlen;
                let score = position_score * coverage_score;
                best = best.max(score);
                if score >= 1.0 {
                    break;
                }
            } else if max_dist > 0 {
                // Fuzzy match — Levenshtein against full word
                let len_diff = (word.len() as isize - tk.len() as isize).unsigned_abs();
                if len_diff <= max_dist {
                    let dist = levenshtein(tk, word);
                    if dist > 0 && dist <= max_dist {
                        let score = match dist {
                            1 => 0.5,
                            2 => 0.25,
                            _ => 0.1,
                        };
                        best = best.max(score);
                    }
                }
            }
        }
        if best > 0.0 {
            matched_count += 1;
        }
        total += best;
    }

    if num_tokens > 1 && matched_count == num_tokens {
        total *= 3.0;
    } else if num_tokens > 1 {
        total *= matched_count as f32 / num_tokens as f32;
    }

    // Heading boost: score tokens against heading words using same position+coverage logic
    // Also split camelCase/hyphenated heading words (e.g. "CodeBlock" → ["code","block"])
    if !heading_words.is_empty() {
        let mut expanded_heading: Vec<String> = Vec::new();
        for hw in &heading_words {
            expanded_heading.push(hw.to_string());
            let parts = split_camel_case(hw);
            if parts.len() > 1 {
                for p in parts {
                    let lower = p.to_lowercase();
                    if lower.len() > 1 {
                        expanded_heading.push(lower);
                    }
                }
            }
        }

        let mut heading_score = 0.0_f32;
        let mut heading_hits = 0usize;
        for tk in tokens {
            let mut best = 0.0_f32;
            for hw in &expanded_heading {
                if let Some(pos) = hw.find(tk.as_str()) {
                    let wlen = hw.len().max(1) as f32;
                    let ps = 1.0 - (pos as f32 / wlen);
                    let cs = tk.len() as f32 / wlen;
                    best = best.max(ps * cs);
                }
            }
            if best > 0.0 {
                heading_hits += 1;
                heading_score += best;
            }
        }
        if heading_hits > 0 {
            let heading_ratio = heading_hits as f32 / num_tokens.max(1) as f32;
            total *= 1.0 + heading_ratio * heading_score * 2.0;
        }
    }

    total
}

/// Get the text for the section containing the given offset (from heading to next heading).
fn get_section_text<'a>(
    text: &'a str,
    headings: &[crate::types::HeadingPos],
    offset: usize,
) -> &'a str {
    // If offset is before the first heading, return intro text only
    if headings.is_empty() || offset < headings[0].offset {
        let end = headings.first().map(|h| h.offset).unwrap_or(text.len());
        return &text[0..end];
    }
    let mut section_start = 0;
    let mut section_end = text.len();
    for (i, h) in headings.iter().enumerate() {
        if h.offset <= offset {
            section_start = h.offset;
            section_end = headings
                .get(i + 1)
                .map(|next| next.offset)
                .unwrap_or(text.len());
        } else {
            break;
        }
    }
    &text[section_start..section_end]
}

/// Find ALL character offsets where any matched term appears in the text.
fn find_all_match_offsets(text: &str, matched_terms: &[String]) -> Vec<usize> {
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
fn context_snippet_at(
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

/// Given a match offset, walk backwards through heading positions to build breadcrumb.
/// Returns (breadcrumb, anchor) where breadcrumb is e.g. ["Page Title", "h2"]
/// and anchor is the closest heading's anchor.
fn resolve_heading_breadcrumb(
    page_title: &str,
    headings: &[crate::types::HeadingPos],
    match_offset: Option<usize>,
) -> (Vec<String>, String) {
    let offset = match match_offset {
        Some(o) => o,
        None => return (vec![], String::new()),
    };

    // Find the last heading whose offset <= match offset
    let mut closest_idx: Option<usize> = None;
    for (i, h) in headings.iter().enumerate() {
        if h.offset <= offset {
            closest_idx = Some(i);
        } else {
            break;
        }
    }

    let closest_idx = match closest_idx {
        Some(i) => i,
        None => return (vec![], String::new()), // Match is before any heading
    };

    let closest = &headings[closest_idx];
    let anchor = closest.anchor.clone();

    // Build breadcrumb: page title, then ancestor headings (lower depth), then closest
    let mut crumbs = vec![page_title.to_string()];
    let closest_depth = closest.depth;

    // Walk backwards to find ancestor headings
    let mut need_depth = closest_depth - 1;
    let mut ancestors = Vec::new();
    for i in (0..closest_idx).rev() {
        if headings[i].depth <= need_depth {
            ancestors.push(headings[i].title.clone());
            if headings[i].depth <= 2 {
                break;
            }
            need_depth = headings[i].depth - 1;
        }
    }
    ancestors.reverse();
    crumbs.extend(ancestors);
    crumbs.push(closest.title.clone());

    (crumbs, anchor)
}

/// Find matching posting keys for a query token.
/// Returns (key, discount, is_fuzzy) tuples. is_fuzzy=true for Levenshtein matches.
fn find_matching_postings<'a>(token: &str, keys: &[&'a String]) -> Vec<(&'a str, f32, bool)> {
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

/// Max edit distance based on term length.
/// Short terms (1-3 chars): 0 (no fuzzy)
/// Medium terms (4-6 chars): 1
/// Longer terms (7+): 2
fn max_edit_distance(len: usize) -> usize {
    if len <= 3 {
        0
    } else if len <= 6 {
        1
    } else {
        2
    }
}

/// Compute Levenshtein edit distance between two strings.
fn levenshtein(a: &str, b: &str) -> usize {
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

fn tokenize(text: &str) -> Vec<String> {
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
fn split_camel_case(s: &str) -> Vec<&str> {
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
    use std::collections::HashMap;

    fn create_test_index() -> LexicalIndex {
        let mut postings = HashMap::new();
        postings.insert(
            "hello".to_string(),
            vec![
                crate::types::Posting {
                    doc_id: 0,
                    score: 2.0,
                },
                crate::types::Posting {
                    doc_id: 1,
                    score: 1.5,
                },
            ],
        );
        postings.insert(
            "world".to_string(),
            vec![crate::types::Posting {
                doc_id: 0,
                score: 1.8,
            }],
        );
        postings.insert(
            "rust".to_string(),
            vec![crate::types::Posting {
                doc_id: 2,
                score: 2.2,
            }],
        );
        postings.insert(
            "block".to_string(),
            vec![crate::types::Posting {
                doc_id: 3,
                score: 1.9,
            }],
        );
        postings.insert(
            "blocks".to_string(),
            vec![crate::types::Posting {
                doc_id: 3,
                score: 1.7,
            }],
        );
        postings.insert(
            "code".to_string(),
            vec![crate::types::Posting {
                doc_id: 3,
                score: 2.1,
            }],
        );

        LexicalIndex {
            postings,
            documents: vec![
                crate::types::DocMetadata {
                    id: 0,
                    title: "Hello World".to_string(),
                    path: "/docs/hello".to_string(),
                    snippet: "A hello world example".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                crate::types::DocMetadata {
                    id: 1,
                    title: "Hello Rust".to_string(),
                    path: "/docs/hello-rust".to_string(),
                    snippet: "Hello from Rust".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                crate::types::DocMetadata {
                    id: 2,
                    title: "Rust Guide".to_string(),
                    path: "/docs/rust".to_string(),
                    snippet: "Learn Rust programming".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                crate::types::DocMetadata {
                    id: 3,
                    title: "Code Blocks".to_string(),
                    path: "/docs/code-blocks".to_string(),
                    snippet: "How to use code blocks".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
            ],
        }
    }

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
    fn test_fuzzy_match_glock_to_block() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "code glock".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert!(!results.is_empty(), "should find results for 'code glock'");
        assert_eq!(results[0].title, "Code Blocks");
    }

    #[test]
    fn test_exact_match_still_works() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "hello".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Hello World");
    }

    #[test]
    fn test_prefix_match_still_works() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "bloc".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert!(!results.is_empty(), "prefix match should work");
    }

    #[test]
    fn test_empty_query() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_no_matches() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "zzzzzzz".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_max_results() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "hello".to_string(),
            max_results: 1,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 1);
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
