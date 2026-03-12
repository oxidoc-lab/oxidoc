//! Semantic (vector) search index generation at build time.
//!
//! The build engine prepares document metadata for semantic search. Actual embedding
//! computation happens client-side in the `oxidoc-search` Wasm crate, which has access
//! to the ML runtime (boostr/numr/splintr).

use crate::config::SearchConfig;
use crate::error::Result;
use std::path::Path;

use super::types::{DocMetadata, PageContent, VectorIndex};

/// Build a semantic search index with document metadata.
///
/// Returns `Ok(Some(index))` if a model is configured (documents prepared for client-side embedding).
/// Returns `Ok(None)` if no model is configured (semantic search disabled).
pub fn build_vector_index(
    pages: &[PageContent],
    config: &SearchConfig,
) -> Result<Option<VectorIndex>> {
    if config.model_path.is_none() {
        return Ok(None);
    }

    let documents: Vec<DocMetadata> = pages
        .iter()
        .enumerate()
        .map(|(idx, page)| DocMetadata {
            id: idx as u32,
            title: page.title.clone(),
            path: format!("/{}", page.slug),
            snippet: super::types::create_snippet(&page.text, 160),
            text: page.text.clone(),
            headings: page.headings.clone(),
        })
        .collect();

    Ok(Some(VectorIndex {
        documents,
        vectors: Vec::new(),
        dimension: 0,
    }))
}

/// Write the vector index to a JSON file.
pub fn write_vector_index(index: &VectorIndex, output_dir: &Path) -> Result<()> {
    let output_path = output_dir.join("search-vectors.json");
    let json = serde_json::to_string(index).map_err(|e| crate::error::OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: std::io::Error::other(e),
    })?;
    std::fs::write(&output_path, json).map_err(|e| crate::error::OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_vector_index_no_model() {
        let config = SearchConfig::default();
        let pages = vec![PageContent {
            title: "Test".to_string(),
            slug: "test".to_string(),
            text: "hello world".to_string(),
            headings: vec![],
        }];

        let result = build_vector_index(&pages, &config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_vector_index_with_model_path() {
        let config = SearchConfig {
            model_path: Some("/model.gguf".to_string()),
            ..SearchConfig::default()
        };
        let pages = vec![PageContent {
            title: "Test".to_string(),
            slug: "test".to_string(),
            text: "hello world".to_string(),
            headings: vec![],
        }];

        let result = build_vector_index(&pages, &config).unwrap();
        let index = result.expect("should produce index when model configured");
        assert_eq!(index.documents.len(), 1);
        assert_eq!(index.documents[0].title, "Test");
    }

    #[test]
    fn test_write_vector_index() {
        let tmp = tempfile::tempdir().unwrap();
        let index = VectorIndex {
            documents: vec![DocMetadata {
                id: 0,
                title: "Test".to_string(),
                path: "/test".to_string(),
                snippet: "test snippet".to_string(),
                text: String::new(),
                headings: vec![],
            }],
            vectors: vec![vec![0.1, 0.2, 0.3]],
            dimension: 3,
        };

        write_vector_index(&index, tmp.path()).unwrap();
        assert!(tmp.path().join("search-vectors.json").exists());

        let json = std::fs::read_to_string(tmp.path().join("search-vectors.json")).unwrap();
        let deserialized: VectorIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.documents.len(), 1);
        assert_eq!(deserialized.vectors.len(), 1);
        assert_eq!(deserialized.dimension, 3);
    }
}
