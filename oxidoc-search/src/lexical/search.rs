use crate::error::SearchResult;
use crate::index::{deserialize_chunk, deserialize_search_metadata};
use crate::types::{
    DocMetadata, LexicalIndex, Posting, SearchMetadata, SearchQuery, SearchResult as DocResult,
    SearchSource,
};
use std::collections::{HashMap, HashSet};

use super::matching::{context_snippet_at, find_all_match_offsets};
use super::resolve::resolve_tokens;
use super::scoring::{
    compute_phrase_boost, get_section_text, resolve_heading_breadcrumb, score_section,
};

pub struct LexicalSearcher {
    documents: Vec<DocMetadata>,
    /// All loaded postings (merged from chunks or from a full index).
    postings: HashMap<String, Vec<Posting>>,
    /// Metadata for chunk-based loading.
    metadata: Option<SearchMetadata>,
}

impl LexicalSearcher {
    /// Load from a full rkyv-serialized LexicalIndex.
    pub fn from_bytes(data: &[u8]) -> SearchResult<Self> {
        let index: LexicalIndex = rkyv::from_bytes::<LexicalIndex, rkyv::rancor::Error>(data)
            .map_err(|e| {
                crate::error::SearchError::IndexLoad(format!(
                    "Failed to deserialize lexical index: {}",
                    e
                ))
            })?;
        Ok(Self {
            documents: index.documents,
            postings: index.postings,
            metadata: None,
        })
    }

    /// Load from SearchMetadata (chunk-based: documents + manifest, no postings yet).
    pub fn from_metadata(data: &[u8]) -> SearchResult<Self> {
        let metadata = deserialize_search_metadata(data)?;
        let documents = metadata.documents.clone();
        Ok(Self {
            documents,
            postings: HashMap::new(),
            metadata: Some(metadata),
        })
    }

    /// Load a chunk's postings into this searcher.
    pub fn load_chunk(&mut self, data: &[u8]) -> SearchResult<()> {
        let chunk: HashMap<String, Vec<Posting>> = deserialize_chunk(data)?;
        self.postings.extend(chunk);
        Ok(())
    }

    /// Get chunk IDs needed for a query (based on 2-char prefix matching).
    pub fn needed_chunk_ids(&self, query: &str) -> Vec<u32> {
        let metadata = match &self.metadata {
            Some(m) => m,
            None => return vec![],
        };

        let tokens: Vec<String> = oxidoc_text::tokenize_query(query)
            .into_iter()
            .flat_map(|(s, r)| if s == r { vec![s] } else { vec![s, r] })
            .collect();
        let mut needed: HashSet<u32> = HashSet::new();

        let prefix_to_chunks: HashMap<&str, Vec<u32>> = {
            let mut map: HashMap<&str, Vec<u32>> = HashMap::new();
            for chunk in &metadata.manifest.chunks {
                for p in &chunk.prefixes {
                    map.entry(p.as_str()).or_default().push(chunk.id);
                }
            }
            map
        };

        for token in &tokens {
            let prefix = match token.char_indices().nth(2) {
                Some((idx, _)) => &token[..idx],
                None => token.as_str(),
            };

            if let Some(chunk_ids) = prefix_to_chunks.get(prefix) {
                needed.extend(chunk_ids);
            }
        }

        let mut ids: Vec<u32> = needed.into_iter().collect();
        ids.sort();
        ids
    }

    pub fn new(index: LexicalIndex) -> Self {
        Self {
            documents: index.documents,
            postings: index.postings,
            metadata: None,
        }
    }

    pub fn search(&self, query: &SearchQuery) -> Vec<DocResult> {
        let text = query.text.trim();
        if text.is_empty() {
            return Vec::new();
        }

        let pairs = oxidoc_text::tokenize_query(text);
        if pairs.is_empty() {
            return Vec::new();
        }
        let tokens: Vec<String> = pairs.iter().map(|(s, _)| s.clone()).collect();
        let raw_tokens: Vec<String> = pairs.iter().map(|(_, r)| r.clone()).collect();

        let resolved = resolve_tokens(&tokens, &raw_tokens, &self.postings);
        let num_tokens = tokens.len();
        let mut all_results: Vec<DocResult> = Vec::new();

        let doc_map: HashMap<u32, &DocMetadata> =
            self.documents.iter().map(|d| (d.id, d)).collect();

        // A doc whose title covers all query tokens is likely the canonical
        // page for the query — boost its results so the dedicated page outranks
        // sibling subsections that merely mention the term. A token counts as
        // covering a title word if it's stem-equal OR a prefix of the title
        // word (so "ve" still matches a title containing "Versioning").
        let doc_title_full_match = |doc: &DocMetadata| -> bool {
            let title_words: Vec<(String, String)> = doc
                .title
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .map(|w| {
                    let lower = w.to_lowercase();
                    let stem = oxidoc_text::stem(&lower);
                    (lower, stem)
                })
                .collect();
            if title_words.is_empty() {
                return false;
            }
            tokens.iter().enumerate().all(|(i, t)| {
                let raw = raw_tokens.get(i).map(|s| s.as_str()).unwrap_or(t.as_str());
                title_words.iter().any(|(lower, stem)| {
                    stem == t || lower.starts_with(t.as_str()) || lower.starts_with(raw)
                })
            })
        };

        for doc_id in &resolved.candidate_docs {
            let doc = match doc_map.get(doc_id) {
                Some(d) => d,
                None => continue,
            };

            let mut terms = resolved
                .doc_matched_keys
                .get(doc_id)
                .cloned()
                .unwrap_or_default();
            terms.sort();
            terms.dedup();

            let mut highlight_terms = resolved
                .doc_fuzzy_keys
                .get(doc_id)
                .cloned()
                .unwrap_or_default();
            highlight_terms.sort();
            highlight_terms.dedup();

            // Compute OR penalty if AND failed
            let and_penalty = if !resolved.use_and {
                let matched_tokens = resolved
                    .per_token_doc_ids
                    .iter()
                    .filter(|ids| ids.contains(doc_id))
                    .count();
                matched_tokens as f32 / num_tokens as f32
            } else {
                1.0
            };

            // Compute phrase boost
            let phrase_boost =
                compute_phrase_boost(&tokens, &resolved.token_postings_for_phrase, *doc_id);

            if doc.text.is_empty() {
                all_results.push(DocResult {
                    title: doc.title.clone(),
                    path: doc.path.clone(),
                    snippet: doc.snippet.clone(),
                    score: 0.1 * and_penalty * phrase_boost,
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

            let title_match = doc_title_full_match(doc);
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
                // Page-title match: boost the intro section (no subheading) strongly
                // so the dedicated page wins over subsections that share the term.
                // Lesser boost for subsections of the same page.
                let title_boost = if title_match {
                    if breadcrumb.is_empty() { 2.0 } else { 1.2 }
                } else {
                    1.0
                };
                let score = score_section(
                    section_text,
                    heading_title,
                    &tokens,
                    &raw_tokens,
                    num_tokens,
                ) * and_penalty
                    * phrase_boost
                    * title_boost;

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
                Posting {
                    doc_id: 0,
                    score: 2.0,
                    positions: vec![0],
                },
                Posting {
                    doc_id: 1,
                    score: 1.5,
                    positions: vec![0],
                },
            ],
        );
        postings.insert(
            "world".to_string(),
            vec![Posting {
                doc_id: 0,
                score: 1.8,
                positions: vec![1],
            }],
        );
        postings.insert(
            "rust".to_string(),
            vec![Posting {
                doc_id: 2,
                score: 2.2,
                positions: vec![0],
            }],
        );
        postings.insert(
            "block".to_string(),
            vec![Posting {
                doc_id: 3,
                score: 1.9,
                positions: vec![1],
            }],
        );
        postings.insert(
            "blocks".to_string(),
            vec![Posting {
                doc_id: 3,
                score: 1.7,
                positions: vec![2],
            }],
        );
        postings.insert(
            "code".to_string(),
            vec![Posting {
                doc_id: 3,
                score: 2.1,
                positions: vec![0],
            }],
        );

        LexicalIndex {
            postings,
            documents: vec![
                DocMetadata {
                    id: 0,
                    title: "Hello World".to_string(),
                    path: "/docs/hello".to_string(),
                    snippet: "A hello world example".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                DocMetadata {
                    id: 1,
                    title: "Hello Rust".to_string(),
                    path: "/docs/hello-rust".to_string(),
                    snippet: "Hello from Rust".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                DocMetadata {
                    id: 2,
                    title: "Rust Guide".to_string(),
                    path: "/docs/rust".to_string(),
                    snippet: "Learn Rust programming".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                DocMetadata {
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
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Hello World"));
        assert!(titles.contains(&"Hello Rust"));
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
    fn test_and_logic_prefers_multi_match() {
        let index = create_test_index();
        let searcher = LexicalSearcher::new(index);
        // "hello world" should prefer doc 0 (has both) over doc 1 (only "hello")
        let query = SearchQuery {
            text: "hello world".to_string(),
            max_results: 10,
        };

        let results = searcher.search(&query);
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Hello World");
    }

    #[test]
    fn test_phrase_boost() {
        let mut postings = HashMap::new();
        postings.insert(
            "hello".to_string(),
            vec![
                Posting {
                    doc_id: 0,
                    score: 1.0,
                    positions: vec![0],
                },
                Posting {
                    doc_id: 1,
                    score: 1.0,
                    positions: vec![5],
                },
            ],
        );
        postings.insert(
            "world".to_string(),
            vec![
                Posting {
                    doc_id: 0,
                    score: 1.0,
                    positions: vec![1],
                }, // adjacent to hello
                Posting {
                    doc_id: 1,
                    score: 1.0,
                    positions: vec![0],
                }, // not adjacent
            ],
        );

        let index = LexicalIndex {
            postings,
            documents: vec![
                DocMetadata {
                    id: 0,
                    title: "Doc A".to_string(),
                    path: "/a".to_string(),
                    snippet: "hello world".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
                DocMetadata {
                    id: 1,
                    title: "Doc B".to_string(),
                    path: "/b".to_string(),
                    snippet: "world then hello".to_string(),
                    text: String::new(),
                    headings: vec![],
                },
            ],
        };

        let searcher = LexicalSearcher::new(index);
        let query = SearchQuery {
            text: "hello world".to_string(),
            max_results: 10,
        };
        let results = searcher.search(&query);

        assert!(!results.is_empty());
        // Doc A should rank higher due to phrase boost (positions 0,1 are adjacent)
        assert_eq!(results[0].title, "Doc A");
    }
}
