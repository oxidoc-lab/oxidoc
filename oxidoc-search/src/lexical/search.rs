use crate::error::SearchResult;
use crate::index::{deserialize_chunk, deserialize_search_metadata};
use crate::records::codec::check_records;
use crate::records::filter::filters_are_empty;
use crate::records::{
    DocRecord, Filter, SearchRecords, decode_records, effective_boost, matches_filters,
};
use crate::types::{
    DocMetadata, LexicalIndex, Posting, SearchMetadata, SearchQuery, SearchResult as DocResult,
};
use std::collections::{HashMap, HashSet};

use super::rank::sort_hits;

/// Keyword searcher over a lexical index, with optional per-document records.
pub struct LexicalSearcher {
    pub(super) documents: Vec<DocMetadata>,
    /// All loaded postings (merged from chunks or from a full index).
    pub(super) postings: HashMap<String, Vec<Posting>>,
    /// Metadata for chunk-based loading.
    metadata: Option<SearchMetadata>,
    /// Ranking data per doc id, from `search-records.bin`.
    pub(super) records: Option<Vec<DocRecord>>,
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
        Ok(Self::new(index))
    }

    /// Load from SearchMetadata (chunk-based: documents + manifest, no postings yet).
    pub fn from_metadata(data: &[u8]) -> SearchResult<Self> {
        let metadata = deserialize_search_metadata(data)?;
        let documents = metadata.documents.clone();
        Ok(Self {
            documents,
            postings: HashMap::new(),
            metadata: Some(metadata),
            records: None,
        })
    }

    /// Creates a searcher over a full in-memory index.
    pub fn new(index: LexicalIndex) -> Self {
        Self {
            documents: index.documents,
            postings: index.postings,
            metadata: None,
            records: None,
        }
    }

    /// Load a chunk's postings into this searcher.
    pub fn load_chunk(&mut self, data: &[u8]) -> SearchResult<()> {
        let chunk: HashMap<String, Vec<Posting>> = deserialize_chunk(data)?;
        self.postings.extend(chunk);
        Ok(())
    }

    /// Decodes `search-records.bin` bytes and enables field weights, boost and tags.
    ///
    /// Fails on an unknown version or on records that do not match the documents.
    pub fn load_records(&mut self, data: &[u8]) -> SearchResult<()> {
        self.set_records(decode_records(data)?)
    }

    /// Enables field weights, boost and tags from decoded records.
    ///
    /// Fails when the records do not match the documents.
    pub fn set_records(&mut self, records: SearchRecords) -> SearchResult<()> {
        check_records(&records, &self.documents)?;
        self.records = Some(records.docs);
        Ok(())
    }

    /// Returns true when records are loaded.
    pub fn has_records(&self) -> bool {
        self.records.is_some()
    }

    /// Returns the number of documents.
    pub fn document_count(&self) -> usize {
        self.documents.len()
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

    /// Runs a keyword query and returns the best `query.max_results` hits.
    ///
    /// `query.filters` limits the candidates. With records loaded, each hit score
    /// is multiplied by the document's clamped boost.
    pub fn search(&self, query: &SearchQuery) -> Vec<DocResult> {
        let mut hits = self.search_hits(&query.text, &query.filters);
        if self.records.is_some() {
            for hit in &mut hits {
                hit.result.score *= self.boost_of(hit.doc_id);
            }
            sort_hits(&mut hits);
        }
        hits.truncate(query.max_results);
        hits.into_iter().map(|hit| hit.result).collect()
    }

    /// Returns the record of `doc_id`, when records are loaded.
    pub(super) fn record(&self, doc_id: u32) -> Option<&DocRecord> {
        self.records.as_ref()?.get(doc_id as usize)
    }

    /// Returns the tags of `doc_id`. Empty without records.
    pub(crate) fn tags_of(&self, doc_id: u32) -> &[(String, String)] {
        self.record(doc_id)
            .map(|r| r.tags.as_slice())
            .unwrap_or_default()
    }

    /// Returns true when `doc_id` passes `filters`.
    ///
    /// Without records no document has tags, so any non-empty filter rejects it.
    pub(crate) fn passes_filters(&self, doc_id: u32, filters: &[Filter]) -> bool {
        filters_are_empty(filters) || matches_filters(self.tags_of(doc_id), filters)
    }

    /// Returns the clamped boost of `doc_id`. 1.0 without records.
    pub(crate) fn boost_of(&self, doc_id: u32) -> f32 {
        self.record(doc_id)
            .map_or(1.0, |r| effective_boost(r.boost))
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
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
            filters: Vec::new(),
        };
        let results = searcher.search(&query);

        assert!(!results.is_empty());
        // Doc A should rank higher due to phrase boost (positions 0,1 are adjacent)
        assert_eq!(results[0].title, "Doc A");
    }
}
