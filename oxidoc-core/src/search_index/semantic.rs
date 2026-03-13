//! Semantic (vector) search index generation at build time.
//!
//! When a GGUF sentence embedding model is configured, the build engine:
//! 1. Loads the model via `EmbeddingPipeline::from_gguf()` (auto-detects tokenizer)
//! 2. Computes embeddings for all document texts using CPU backend
//! 3. Writes pre-computed vectors to `search-vectors.json`
//!
//! At runtime, the oxidoc-search Wasm crate loads these vectors and the same
//! GGUF model, computes only the query embedding, then does cosine similarity.

use crate::config::SearchConfig;
use crate::error::{OxidocError, Result};
use std::path::Path;

use super::types::{DocMetadata, PageContent, VectorIndex};

/// Build a semantic search index with pre-computed document embeddings.
///
/// The GGUF file contains model weights, config, AND tokenizer vocab — no
/// separate tokenizer configuration needed.
///
/// `bundled_model` is the default embedded model bytes from the CLI binary.
/// `config.model_path` overrides it with a custom model from disk.
///
/// Returns `Ok(Some(index))` if semantic is enabled and embeddings succeed.
/// Returns `Ok(None)` if semantic search is disabled.
pub fn build_vector_index(
    pages: &[PageContent],
    config: &SearchConfig,
    bundled_model: Option<&[u8]>,
) -> Result<Option<VectorIndex>> {
    if !config.semantic {
        return Ok(None);
    }

    use boostr::format::gguf::Gguf;
    use boostr::model::encoder::EmbeddingPipeline;
    use numr::runtime::cpu::{CpuClient, CpuDevice, CpuRuntime};

    let device = CpuDevice::new();
    let client = CpuClient::new(device.clone());

    // Load model: prefer model_path override, fall back to bundled
    let mut gguf = if let Some(path) = config.model_path.as_ref() {
        tracing::info!(model = %path, "Loading custom sentence embedding model");
        Gguf::open(path).map_err(|e| {
            OxidocError::Search(format!("Failed to open GGUF model '{}': {}", path, e))
        })?
    } else if let Some(bytes) = bundled_model {
        tracing::info!("Loading bundled sentence embedding model");
        Gguf::from_bytes(bytes.to_vec()).map_err(|e| {
            OxidocError::Search(format!("Failed to parse bundled GGUF model: {}", e))
        })?
    } else {
        return Err(OxidocError::Search(
            "Semantic search enabled but no model available (no bundled model or model_path)"
                .to_string(),
        ));
    };

    let pipeline = EmbeddingPipeline::<CpuRuntime, _>::from_gguf(&mut gguf, device)
        .map_err(|e| OxidocError::Search(format!("Failed to load embedding model: {}", e)))?;

    let dimension = pipeline.config().hidden_size;

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

    let texts: Vec<&str> = pages.iter().map(|p| p.text.as_str()).collect();

    tracing::info!(
        docs = texts.len(),
        dim = dimension,
        "Computing document embeddings"
    );

    let vectors = if texts.is_empty() {
        Vec::new()
    } else {
        pipeline
            .embed_texts(&client, &texts)
            .map_err(|e| OxidocError::Search(format!("Failed to compute embeddings: {}", e)))?
    };

    tracing::info!(docs = vectors.len(), "Document embeddings computed");

    Ok(Some(VectorIndex {
        documents,
        vectors,
        dimension,
    }))
}

/// Write the vector index to a JSON file.
pub fn write_vector_index(index: &VectorIndex, output_dir: &Path) -> Result<()> {
    let output_path = output_dir.join("search-vectors.json");
    let json = serde_json::to_string(index).map_err(|e| OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: std::io::Error::other(e),
    })?;
    std::fs::write(&output_path, json).map_err(|e| OxidocError::FileWrite {
        path: output_path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

/// Write the GGUF model to the output directory so the browser can fetch it.
pub fn write_model_to_output(
    config: &SearchConfig,
    bundled_model: Option<&[u8]>,
    output_dir: &Path,
) -> Result<()> {
    let dest = output_dir.join("search-model.gguf");

    if let Some(path) = config.model_path.as_ref() {
        std::fs::copy(Path::new(path), &dest).map_err(|e| OxidocError::FileWrite {
            path: dest.display().to_string(),
            source: e,
        })?;
    } else if let Some(bytes) = bundled_model {
        std::fs::write(&dest, bytes).map_err(|e| OxidocError::FileWrite {
            path: dest.display().to_string(),
            source: e,
        })?;
    } else {
        return Ok(());
    }

    tracing::info!("Wrote embedding model to output directory");
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

        let result = build_vector_index(&pages, &config, None).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_build_vector_index_semantic_disabled() {
        let config = SearchConfig {
            semantic: false,
            ..SearchConfig::default()
        };
        let pages = vec![PageContent {
            title: "Test".to_string(),
            slug: "test".to_string(),
            text: "hello world".to_string(),
            headings: vec![],
        }];
        // Even with bundled bytes, returns None when semantic=false
        let result = build_vector_index(&pages, &config, Some(b"fake")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    #[ignore] // Requires OXIDOC_TEST_MODEL env var pointing to a GGUF embedding model
    fn test_build_vector_index_with_bundled_model() {
        let model_path = std::env::var("OXIDOC_TEST_MODEL")
            .expect("Set OXIDOC_TEST_MODEL to a GGUF embedding model path");

        let model_bytes = std::fs::read(&model_path).unwrap();
        let config = SearchConfig {
            semantic: true,
            ..SearchConfig::default()
        };
        let pages = vec![
            PageContent {
                title: "Getting Started".to_string(),
                slug: "getting-started".to_string(),
                text: "Install the CLI tool and create your first project".to_string(),
                headings: vec![],
            },
            PageContent {
                title: "API Reference".to_string(),
                slug: "api-reference".to_string(),
                text: "The REST API provides endpoints for user management".to_string(),
                headings: vec![],
            },
        ];

        let result = build_vector_index(&pages, &config, Some(&model_bytes)).unwrap();
        let index = result.expect("Should produce vector index");
        assert_eq!(index.documents.len(), 2);
        assert_eq!(index.vectors.len(), 2);
        assert!(index.dimension > 0);
        // Each vector should match the dimension
        for vec in &index.vectors {
            assert_eq!(vec.len(), index.dimension);
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
