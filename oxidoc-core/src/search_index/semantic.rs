//! Semantic (vector) search index generation at build time.
//!
//! When a GGUF embedding model is configured, the build engine:
//! 1. Loads the model via boostr (CPU backend)
//! 2. Computes embeddings for all document texts
//! 3. Writes pre-computed vectors to `search-vectors.json`
//!
//! At runtime, the oxidoc-search Wasm crate loads these vectors and computes
//! only the query embedding, then does cosine similarity for semantic search.

use crate::config::SearchConfig;
use crate::error::{OxidocError, Result};
use std::path::Path;

use super::types::{DocMetadata, PageContent, VectorIndex};

/// Build a semantic search index with pre-computed document embeddings.
///
/// Returns `Ok(Some(index))` if a model is configured and embeddings succeed.
/// Returns `Ok(None)` if no model is configured (semantic search disabled).
pub fn build_vector_index(
    pages: &[PageContent],
    config: &SearchConfig,
) -> Result<Option<VectorIndex>> {
    let model_path = match config.model_path.as_ref() {
        Some(p) => p,
        None => return Ok(None),
    };

    let tokenizer_name = config.tokenizer.as_deref().unwrap_or("cl100k_base");

    tracing::info!(model = %model_path, tokenizer = %tokenizer_name, "Loading embedding model");

    // Load tokenizer
    let tokenizer = splintr::from_pretrained(tokenizer_name).map_err(|e| {
        OxidocError::Search(format!(
            "Failed to load tokenizer '{}': {}",
            tokenizer_name, e
        ))
    })?;

    // Set up CPU runtime
    use numr::runtime::cpu::{CpuClient, CpuDevice};
    let device = CpuDevice::new();
    let client = CpuClient::new(device.clone());

    // Load GGUF model
    use boostr::format::gguf::Gguf;
    let mut gguf = Gguf::open(model_path).map_err(|e| {
        OxidocError::Search(format!("Failed to open GGUF model '{}': {}", model_path, e))
    })?;

    // Extract encoder config from GGUF metadata
    use boostr::model::encoder::{EmbeddingPipeline, Encoder, EncoderConfig, Pooling};
    use numr::runtime::cpu::CpuRuntime;

    let encoder_config = EncoderConfig::from_gguf_metadata(gguf.metadata()).map_err(|e| {
        OxidocError::Search(format!("Failed to read encoder config from GGUF: {}", e))
    })?;

    let dimension = encoder_config.hidden_size;

    // Load encoder weights
    let encoder = Encoder::from_weights(encoder_config, Pooling::Mean, |name| {
        gguf.load_tensor_f32::<CpuRuntime>(name, &device)
    })
    .map_err(|e| OxidocError::Search(format!("Failed to load encoder weights: {}", e)))?;

    // Create embedding pipeline
    let pipeline = EmbeddingPipeline::new(encoder, tokenizer, device);

    // Compute embeddings for all documents
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

    tracing::info!(docs = texts.len(), "Computing document embeddings");

    let vectors = if texts.is_empty() {
        Vec::new()
    } else {
        pipeline
            .embed_texts(&client, &texts)
            .map_err(|e| OxidocError::Search(format!("Failed to compute embeddings: {}", e)))?
    };

    tracing::info!(
        docs = vectors.len(),
        dim = dimension,
        "Document embeddings computed"
    );

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

/// Copy the GGUF model file to the output directory so the browser can fetch it.
pub fn copy_model_to_output(config: &SearchConfig, output_dir: &Path) -> Result<()> {
    let model_path = match config.model_path.as_ref() {
        Some(p) => Path::new(p),
        None => return Ok(()),
    };

    let dest = output_dir.join("search-model.gguf");
    std::fs::copy(model_path, &dest).map_err(|e| OxidocError::FileWrite {
        path: dest.display().to_string(),
        source: e,
    })?;

    tracing::info!("Copied embedding model to output directory");
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
    fn test_copy_model_no_path() {
        let config = SearchConfig::default();
        let tmp = tempfile::tempdir().unwrap();
        copy_model_to_output(&config, tmp.path()).unwrap();
        // Should be a no-op
        assert!(!tmp.path().join("search-model.gguf").exists());
    }
}
