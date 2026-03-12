use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A heading position within a document's text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingPos {
    pub title: String,
    pub anchor: String,
    pub depth: u8,
    /// Character offset in the document's `text` field where this heading's content starts.
    pub offset: usize,
}

/// Metadata for a searchable document (one per page).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub snippet: String,
    /// Full plain text content for search and snippet extraction.
    #[serde(default)]
    pub text: String,
    /// Heading positions within the text, for locating which section a match falls in.
    #[serde(default)]
    pub headings: Vec<HeadingPos>,
}

/// A posting entry in the inverted index (term -> doc with score).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    pub doc_id: u32,
    pub score: f32,
}

/// Lexical (inverted) index for keyword-based search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalIndex {
    pub postings: HashMap<String, Vec<Posting>>,
    pub documents: Vec<DocMetadata>,
}

/// Semantic (vector) index for embedding-based search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    pub documents: Vec<DocMetadata>,
    pub vectors: Vec<Vec<f32>>,
    pub dimension: usize,
}

/// Plain text content extracted from a page (one per page).
#[derive(Debug, Clone)]
pub struct PageContent {
    pub title: String,
    pub slug: String,
    pub text: String,
    /// Heading positions within the text.
    pub headings: Vec<HeadingPos>,
}

/// Create a snippet from text (first N characters).
pub fn create_snippet(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_snippet_truncates() {
        let text = "This is a very long piece of text that should be truncated";
        let snippet = create_snippet(text, 30);
        assert!(snippet.len() <= 33);
        assert!(snippet.ends_with('…'));
    }

    #[test]
    fn test_create_snippet_short_text() {
        assert_eq!(create_snippet("Short", 30), "Short");
    }

    #[test]
    fn test_doc_metadata_serialization() {
        let meta = DocMetadata {
            id: 1,
            title: "Test Page".to_string(),
            path: "/docs/test".to_string(),
            snippet: "This is a test".to_string(),
            text: String::new(),
            headings: vec![],
        };

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
            documents: vec![DocMetadata {
                id: 0,
                title: "Doc1".to_string(),
                path: "/doc1".to_string(),
                snippet: "snippet1".to_string(),
                text: String::new(),
                headings: vec![],
            }],
        };
        index.postings.insert(
            "hello".to_string(),
            vec![Posting {
                doc_id: 0,
                score: 0.95,
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
            documents: vec![DocMetadata {
                id: 0,
                title: "Doc1".to_string(),
                path: "/doc1".to_string(),
                snippet: "snippet1".to_string(),
                text: String::new(),
                headings: vec![],
            }],
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
