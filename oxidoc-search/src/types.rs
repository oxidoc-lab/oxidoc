use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub path: String,
    pub snippet: String,
    pub score: f32,
    pub source: SearchSource,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SearchSource {
    Semantic,
    Lexical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocMetadata {
    pub id: u32,
    pub title: String,
    pub path: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalIndex {
    pub postings: HashMap<String, Vec<Posting>>,
    pub documents: Vec<DocMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    pub doc_id: u32,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    pub documents: Vec<DocMetadata>,
    pub vectors: Vec<Vec<f32>>,
    pub dimension: usize,
}

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
}
