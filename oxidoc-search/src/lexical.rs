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

    pub fn search(&self, query: &SearchQuery) -> Vec<DocResult> {
        let text = query.text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        let tokens = tokenize(text);
        if tokens.is_empty() {
            return Vec::new();
        }

        let mut doc_scores: HashMap<u32, f32> = HashMap::new();
        let mut doc_hit_counts: HashMap<u32, usize> = HashMap::new();

        for token in &tokens {
            if let Some(postings) = self.index.postings.get(token) {
                for posting in postings {
                    let score = doc_scores.entry(posting.doc_id).or_insert(0.0);
                    *score += posting.score;
                    let hits = doc_hit_counts.entry(posting.doc_id).or_insert(0);
                    *hits += 1;
                }
            }
        }

        let mut results: Vec<(u32, f32)> = doc_scores
            .into_iter()
            .filter(|(doc_id, _)| doc_hit_counts.get(doc_id).copied().unwrap_or(0) > 0)
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        results
            .into_iter()
            .take(query.max_results)
            .filter_map(|(doc_id, score)| {
                self.index
                    .documents
                    .iter()
                    .find(|d| d.id == doc_id)
                    .map(|doc| DocResult {
                        title: doc.title.clone(),
                        path: doc.path.clone(),
                        snippet: doc.snippet.clone(),
                        score,
                        source: SearchSource::Lexical,
                    })
            })
            .collect()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|token| token.len() > 1)
        .map(|token| {
            token
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect::<String>()
        })
        .filter(|token| !token.is_empty())
        .collect()
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

        LexicalIndex {
            postings,
            documents: vec![
                crate::types::DocMetadata {
                    id: 0,
                    title: "Hello World".to_string(),
                    path: "/docs/hello".to_string(),
                    snippet: "A hello world example".to_string(),
                },
                crate::types::DocMetadata {
                    id: 1,
                    title: "Hello Rust".to_string(),
                    path: "/docs/hello-rust".to_string(),
                    snippet: "Hello from Rust".to_string(),
                },
                crate::types::DocMetadata {
                    id: 2,
                    title: "Rust Guide".to_string(),
                    path: "/docs/rust".to_string(),
                    snippet: "Learn Rust programming".to_string(),
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
    fn test_single_term_search() {
        let index = create_test_index();
        let searcher = LexicalSearcher { index };
        let query = SearchQuery {
            text: "hello".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Hello World");
        assert_eq!(results[1].title, "Hello Rust");
    }

    #[test]
    fn test_multi_term_search() {
        let index = create_test_index();
        let searcher = LexicalSearcher { index };
        let query = SearchQuery {
            text: "hello world".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "Hello World");
        assert_eq!(results[1].title, "Hello Rust");
    }

    #[test]
    fn test_empty_query() {
        let index = create_test_index();
        let searcher = LexicalSearcher { index };
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
        let searcher = LexicalSearcher { index };
        let query = SearchQuery {
            text: "nonexistent".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_max_results() {
        let index = create_test_index();
        let searcher = LexicalSearcher { index };
        let query = SearchQuery {
            text: "hello rust".to_string(),
            max_results: 1,
        };

        let results = searcher.search(&query);
        assert_eq!(results.len(), 1);
    }
}
