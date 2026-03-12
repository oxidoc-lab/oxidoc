use crate::types::{LexicalIndex, VectorIndex};
use serde_json;

pub fn deserialize_lexical_index(data: &[u8]) -> crate::error::SearchResult<LexicalIndex> {
    serde_json::from_slice(data).map_err(|e| {
        crate::error::SearchError::IndexLoad(format!("Failed to deserialize lexical index: {}", e))
    })
}

pub fn deserialize_vector_index(data: &[u8]) -> crate::error::SearchResult<VectorIndex> {
    serde_json::from_slice(data).map_err(|e| {
        crate::error::SearchError::IndexLoad(format!("Failed to deserialize vector index: {}", e))
    })
}

pub fn serialize_lexical_index(index: &LexicalIndex) -> crate::error::SearchResult<Vec<u8>> {
    serde_json::to_vec(index).map_err(|e| {
        crate::error::SearchError::Serialization(format!(
            "Failed to serialize lexical index: {}",
            e
        ))
    })
}

pub fn serialize_vector_index(index: &VectorIndex) -> crate::error::SearchResult<Vec<u8>> {
    serde_json::to_vec(index).map_err(|e| {
        crate::error::SearchError::Serialization(format!("Failed to serialize vector index: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DocMetadata, Posting};
    use std::collections::HashMap;

    #[test]
    fn test_lexical_index_roundtrip() {
        let mut postings = HashMap::new();
        postings.insert(
            "hello".to_string(),
            vec![Posting {
                doc_id: 0,
                score: 1.5,
            }],
        );

        let index = LexicalIndex {
            postings,
            documents: vec![DocMetadata {
                id: 0,
                title: "Test".to_string(),
                path: "/test".to_string(),
                snippet: "Hello world".to_string(),
                text: String::new(),
                headings: vec![],
            }],
        };

        let serialized = serialize_lexical_index(&index).unwrap();
        let deserialized = deserialize_lexical_index(&serialized).unwrap();

        assert_eq!(index.documents.len(), deserialized.documents.len());
        assert_eq!(index.postings.len(), deserialized.postings.len());
    }

    #[test]
    fn test_vector_index_roundtrip() {
        let index = VectorIndex {
            documents: vec![DocMetadata {
                id: 0,
                title: "Test".to_string(),
                path: "/test".to_string(),
                snippet: "Hello world".to_string(),
                text: String::new(),
                headings: vec![],
            }],
            vectors: vec![vec![0.1, 0.2, 0.3]],
            dimension: 3,
        };

        let serialized = serialize_vector_index(&index).unwrap();
        let deserialized = deserialize_vector_index(&serialized).unwrap();

        assert_eq!(index.documents.len(), deserialized.documents.len());
        assert_eq!(index.dimension, deserialized.dimension);
        assert_eq!(index.vectors.len(), deserialized.vectors.len());
    }
}
