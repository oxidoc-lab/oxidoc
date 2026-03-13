use crate::error::SearchResult;
use crate::index::deserialize_lexical_index;
use crate::types::{LexicalIndex, SearchQuery, SearchResult as DocResult, SearchSource};
use std::collections::HashMap;

use super::matching::{
    context_snippet_at, find_all_match_offsets, find_matching_postings, tokenize,
};
use super::scoring::{get_section_text, resolve_heading_breadcrumb, score_section};

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
}
