//! Semantic (vector) index for site pages, built by `oxidoc_search::builder`.
//!
//! This module picks the model from the search config. `oxidoc_search` embeds the
//! pages, writes `search-vectors.bin`, and copies the model for the browser.

use crate::config::SearchConfig;
use crate::error::{OxidocError, Result};
use oxidoc_search::builder::{self, EmbeddingModel};
use oxidoc_search::{QuantVectors, Record};
use std::path::Path;

/// Returns the configured model: `config.model_path` first, then the bundled bytes.
fn select_model<'a>(
    config: &'a SearchConfig,
    bundled_model: Option<&'a [u8]>,
) -> Option<EmbeddingModel<'a>> {
    match (config.model_path.as_ref(), bundled_model) {
        (Some(path), _) => Some(EmbeddingModel::File(Path::new(path))),
        (None, Some(bytes)) => Some(EmbeddingModel::Bytes(bytes)),
        (None, None) => None,
    }
}

/// Builds a semantic search index with pre-computed embeddings of `records`.
///
/// `bundled_model` is the default embedded model bytes from the CLI binary.
/// `config.model_path` overrides it with a custom model from disk.
///
/// Returns `Ok(None)` when semantic search is disabled.
/// Returns an error when it is enabled and no model is available.
pub fn build_vector_index(
    records: &[Record],
    config: &SearchConfig,
    bundled_model: Option<&[u8]>,
) -> Result<Option<QuantVectors>> {
    if !config.semantic {
        return Ok(None);
    }

    let model = select_model(config, bundled_model).ok_or_else(|| {
        OxidocError::Search(
            "Semantic search enabled but no model available (no bundled model or model_path)"
                .to_string(),
        )
    })?;
    match model {
        EmbeddingModel::File(path) => {
            tracing::info!(model = %path.display(), "Loading custom sentence embedding model");
        }
        EmbeddingModel::Bytes(_) => tracing::info!("Loading bundled sentence embedding model"),
    }

    tracing::info!(docs = records.len(), "Computing document embeddings");
    let index = builder::build_vectors(records, model)?;
    tracing::info!(docs = index.len(), "Document embeddings computed");

    Ok(Some(index))
}

/// Writes the configured GGUF model to the output directory so the browser can fetch it.
///
/// Writes nothing when no model is configured or bundled.
pub fn write_model_to_output(
    config: &SearchConfig,
    bundled_model: Option<&[u8]>,
    output_dir: &Path,
) -> Result<()> {
    let Some(model) = select_model(config, bundled_model) else {
        return Ok(());
    };
    builder::write_model(model, output_dir)?;
    tracing::info!("Wrote embedding model to output directory");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxidoc_search::Field;

    fn records() -> Vec<Record> {
        let mut record = Record::new("test", "/test", "Test");
        record.fields.push(Field::new("body", "hello world"));
        vec![record]
    }

    #[test]
    fn test_build_vector_index_no_model() {
        let config = SearchConfig::default();
        let result = build_vector_index(&records(), &config, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_vector_index_semantic_disabled() {
        let config = SearchConfig {
            semantic: false,
            ..SearchConfig::default()
        };
        // Even with bundled bytes, returns None when semantic=false
        let result = build_vector_index(&records(), &config, Some(b"fake")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_vector_index_enabled_without_model_errors() {
        let config = SearchConfig {
            semantic: true,
            ..SearchConfig::default()
        };
        let err = build_vector_index(&records(), &config, None).unwrap_err();
        assert!(matches!(err, OxidocError::Search(_)));
    }

    #[test]
    fn test_build_vector_index_bad_model_maps_to_search_error() {
        let config = SearchConfig {
            semantic: true,
            ..SearchConfig::default()
        };
        match build_vector_index(&records(), &config, Some(b"not gguf")) {
            Err(OxidocError::Search(message)) => {
                assert!(message.starts_with("Failed to parse bundled GGUF model"));
            }
            other => panic!("expected a search error, got {other:?}"),
        }
    }

    #[test]
    fn test_write_model_no_source() {
        let config = SearchConfig::default();
        let tmp = tempfile::tempdir().unwrap();
        write_model_to_output(&config, None, tmp.path()).unwrap();
        assert!(!tmp.path().join("search-model.gguf").exists());
    }

    #[test]
    fn test_write_model_from_bundled() {
        let config = SearchConfig::default();
        let tmp = tempfile::tempdir().unwrap();
        let fake_model = b"fake gguf data";
        write_model_to_output(&config, Some(fake_model), tmp.path()).unwrap();
        let written = std::fs::read(tmp.path().join("search-model.gguf")).unwrap();
        assert_eq!(written, fake_model);
    }
}
