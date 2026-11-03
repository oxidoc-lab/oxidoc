use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A heading position within a document's text.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct HeadingPos {
    /// Plain text of the heading.
    pub title: String,
    /// Fragment id of the heading, without `#`.
    pub anchor: String,
    /// Heading level, as in `<h2>` = 2.
    pub depth: u8,
    /// Byte offset in the document's `text` field where this heading's content starts.
    pub offset: usize,
}

/// One ranked hit returned to the search UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document title.
    pub title: String,
    /// Document link target.
    pub path: String,
    /// Text shown under the title.
    pub snippet: String,
    /// Rank score. Higher ranks first.
    pub score: f32,
    /// Ranker that produced the hit.
    pub source: SearchSource,
    /// Breadcrumb trail: ["Page Title", "h2", "h3"] — last entry is the closest heading.
    #[serde(default)]
    pub breadcrumb: Vec<String>,
    /// Anchor for the closest heading (for linking to the right section).
    #[serde(default)]
    pub anchor: String,
    /// Terms that actually matched (for highlighting).
    #[serde(default)]
    pub highlight_terms: Vec<String>,
}

/// Ranker that produced a [`SearchResult`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SearchSource {
    /// Embedding similarity.
    Semantic,
    /// BM25 keyword match.
    Lexical,
}

/// A search request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Query text as typed.
    pub text: String,
    /// Maximum number of results to return.
    pub max_results: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            max_results: 10,
        }
    }
}

/// Metadata for a searchable document (one per page).
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct DocMetadata {
    /// Doc id. Equals the document's index in the documents list.
    pub id: u32,
    /// Document title.
    pub title: String,
    /// Document link target.
    pub path: String,
    /// Leading text of the document, for display.
    pub snippet: String,
    /// Full plain text content for search and snippet extraction.
    #[serde(default)]
    pub text: String,
    /// Heading positions within the text, for locating which section a match falls in.
    #[serde(default)]
    pub headings: Vec<HeadingPos>,
}

/// Lexical (inverted) index for keyword-based search.
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct LexicalIndex {
    /// Postings per term, sorted by score descending, then doc id.
    pub postings: HashMap<String, Vec<Posting>>,
    /// Documents in doc id order.
    pub documents: Vec<DocMetadata>,
}

/// A posting entry in the inverted index (term -> doc with score + positions).
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct Posting {
    /// Document that holds the term.
    pub doc_id: u32,
    /// BM25 score of the term in the document.
    pub score: f32,
    /// Token index positions where this term appears in the document.
    #[serde(default)]
    pub positions: Vec<u32>,
}

/// Semantic (vector) index for embedding-based search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    /// Documents in doc id order.
    pub documents: Vec<DocMetadata>,
    /// One embedding per document, in doc id order.
    pub vectors: Vec<Vec<f32>>,
    /// Length of every embedding.
    pub dimension: usize,
}

/// Entry in the chunk manifest describing which term prefixes a chunk covers.
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct ChunkEntry {
    /// Chunk id. The chunk file is `search-chunk-{id}.bin`.
    pub id: u32,
    /// Term prefixes the chunk holds.
    pub prefixes: Vec<String>,
}

/// Manifest listing all chunks and their prefix coverage.
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct ChunkManifest {
    /// Chunks in id order.
    pub chunks: Vec<ChunkEntry>,
}

/// Metadata file loaded at init time (documents + manifest, no postings).
#[derive(
    Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize,
)]
#[rkyv(crate = rkyv)]
pub struct SearchMetadata {
    /// Documents in doc id order.
    pub documents: Vec<DocMetadata>,
    /// Chunk list for lazy postings loads.
    pub manifest: ChunkManifest,
}

/// Postings for a single chunk (term -> postings list).
pub type ChunkPostings = HashMap<String, Vec<Posting>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            title: "Test Page".to_string(),
            path: "/docs/test".to_string(),
            snippet: "This is a test snippet".to_string(),
            score: 0.95,
            source: SearchSource::Semantic,
            breadcrumb: vec![],
            anchor: String::new(),
            highlight_terms: Vec::new(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.title, deserialized.title);
        assert_eq!(result.path, deserialized.path);
        assert_eq!(result.snippet, deserialized.snippet);
        assert!((result.score - deserialized.score).abs() < 0.001);
        assert_eq!(result.source, deserialized.source);
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.text, "");
        assert_eq!(query.max_results, 10);
    }

    fn doc(id: u32, title: &str, path: &str, snippet: &str) -> DocMetadata {
        DocMetadata {
            id,
            title: title.to_string(),
            path: path.to_string(),
            snippet: snippet.to_string(),
            text: String::new(),
            headings: vec![],
        }
    }

    #[test]
    fn test_doc_metadata_serialization() {
        let meta = doc(1, "Test Page", "/docs/test", "This is a test");

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: DocMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.id, deserialized.id);
        assert_eq!(meta.title, deserialized.title);
        assert_eq!(meta.path, deserialized.path);
        assert_eq!(meta.snippet, deserialized.snippet);
    }

    #[test]
    fn test_lexical_index_serialization() {
        let mut index = LexicalIndex {
            postings: HashMap::new(),
            documents: vec![doc(0, "Doc1", "/doc1", "snippet1")],
        };
        index.postings.insert(
            "hello".to_string(),
            vec![Posting {
                doc_id: 0,
                score: 0.95,
                positions: vec![0, 5],
            }],
        );

        let json = serde_json::to_string(&index).unwrap();
        let deserialized: LexicalIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.documents.len(), 1);
        assert_eq!(deserialized.postings.len(), 1);
        assert!((deserialized.postings["hello"][0].score - 0.95).abs() < 0.001);
    }

    #[test]
    fn test_vector_index_serialization() {
        let index = VectorIndex {
            documents: vec![doc(0, "Doc1", "/doc1", "snippet1")],
            vectors: vec![vec![0.1, 0.2, 0.3]],
            dimension: 3,
        };

        let json = serde_json::to_string(&index).unwrap();
        let deserialized: VectorIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.documents.len(), 1);
        assert_eq!(deserialized.vectors.len(), 1);
        assert_eq!(deserialized.dimension, 3);
        assert!((deserialized.vectors[0][0] - 0.1).abs() < 0.001);
    }
}
