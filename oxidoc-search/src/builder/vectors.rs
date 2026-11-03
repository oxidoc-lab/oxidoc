//! Semantic vector index built from records with a GGUF embedding model.
//!
//! The builder embeds every record body on the CPU. The browser loads the same
//! model, embeds only the query, and ranks the stored vectors by cosine similarity.

use std::path::{Path, PathBuf};

use boostr::format::gguf::Gguf;
use boostr::model::encoder::EmbeddingPipeline;
use numr::runtime::cpu::{CpuClient, CpuDevice, CpuRuntime};

use super::document::doc_metadata;
use crate::embed_input::embed_each;
use crate::error::SearchError;
use crate::record::Record;
use crate::types::{DocMetadata, VectorIndex};

/// File name of the vector index.
pub const VECTORS_FILE: &str = "search-vectors.json";
/// File name of the model copy the browser fetches.
pub const MODEL_FILE: &str = "search-model.gguf";

/// Source of a GGUF sentence embedding model.
///
/// The GGUF file holds the weights, the config and the tokenizer vocab.
#[derive(Debug, Clone, Copy)]
pub enum EmbeddingModel<'a> {
    /// A model file on disk.
    File(&'a Path),
    /// Model bytes in memory.
    Bytes(&'a [u8]),
}

/// Embeds each record body and returns the vector index.
///
/// Each body is embedded alone, cut to the model's maximum sequence length.
/// A body with no tokens gets a zero vector, and semantic search never returns it.
/// Doc ids follow input order. Documents carry the same metadata as the lexical index.
pub fn build_vectors(
    records: &[Record],
    model: EmbeddingModel<'_>,
) -> Result<VectorIndex, SearchError> {
    let device = CpuDevice::new();
    let client = CpuClient::new(device.clone());

    let mut gguf = match model {
        EmbeddingModel::File(path) => Gguf::open(path).map_err(|e| {
            SearchError::ModelLoad(format!(
                "Failed to open GGUF model '{}': {}",
                path.display(),
                e
            ))
        })?,
        EmbeddingModel::Bytes(bytes) => Gguf::from_bytes(bytes.to_vec()).map_err(|e| {
            SearchError::ModelLoad(format!("Failed to parse bundled GGUF model: {}", e))
        })?,
    };

    let pipeline = EmbeddingPipeline::<CpuRuntime>::from_gguf(&mut gguf, device.clone())
        .map_err(|e| SearchError::ModelLoad(format!("Failed to load embedding model: {}", e)))?;
    let dimension = pipeline.config().hidden_size;

    let bodies: Vec<String> = records.iter().map(Record::body).collect();
    let texts: Vec<&str> = bodies.iter().map(String::as_str).collect();
    let vectors = embed_each(&pipeline, &client, &texts)
        .map_err(|e| SearchError::Embedding(format!("Failed to compute embeddings: {}", e)))?;

    let documents: Vec<DocMetadata> = records
        .iter()
        .zip(bodies)
        .enumerate()
        .map(|(idx, (record, body))| doc_metadata(idx as u32, record, body))
        .collect();

    Ok(VectorIndex {
        documents,
        vectors,
        dimension,
    })
}

/// Writes `index` to `search-vectors.json` in `dir`. `dir` must exist.
pub fn write_vectors(index: &VectorIndex, dir: &Path) -> Result<(), SearchError> {
    let path = dir.join(VECTORS_FILE);
    let json = serde_json::to_string(index).map_err(|e| SearchError::Encode {
        path: path.clone(),
        message: e.to_string(),
    })?;
    std::fs::write(&path, json).map_err(|source| SearchError::Write { path, source })
}

/// Writes the model to `search-model.gguf` in `dir`, so the browser can fetch it.
pub fn write_model(model: EmbeddingModel<'_>, dir: &Path) -> Result<(), SearchError> {
    let path: PathBuf = dir.join(MODEL_FILE);
    let result = match model {
        EmbeddingModel::File(src) => std::fs::copy(src, &path).map(|_| ()),
        EmbeddingModel::Bytes(bytes) => std::fs::write(&path, bytes),
    };
    result.map_err(|source| SearchError::Write { path, source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Field;

    fn record(slug: &str, text: &str) -> Record {
        let mut record = Record::new(slug, format!("/{slug}"), slug);
        record.fields.push(Field::new("body", text));
        record
    }

    #[test]
    fn write_model_from_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let fake_model = b"fake gguf data";
        write_model(EmbeddingModel::Bytes(fake_model), tmp.path()).unwrap();
        let written = std::fs::read(tmp.path().join(MODEL_FILE)).unwrap();
        assert_eq!(written, fake_model);
    }

    #[test]
    fn write_model_copies_file() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("model.gguf");
        std::fs::write(&src, b"model file").unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();
        write_model(EmbeddingModel::File(&src), &out).unwrap();
        assert_eq!(std::fs::read(out.join(MODEL_FILE)).unwrap(), b"model file");
    }

    #[test]
    fn write_vectors_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let index = VectorIndex {
            documents: Vec::new(),
            vectors: vec![vec![0.5, 0.25]],
            dimension: 2,
        };
        write_vectors(&index, tmp.path()).unwrap();
        let bytes = std::fs::read(tmp.path().join(VECTORS_FILE)).unwrap();
        let back: VectorIndex = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.dimension, 2);
        assert_eq!(back.vectors, vec![vec![0.5, 0.25]]);
    }

    #[test]
    fn build_vectors_rejects_bad_model_bytes() {
        let result = build_vectors(&[record("a", "text")], EmbeddingModel::Bytes(b"not gguf"));
        assert!(matches!(result, Err(SearchError::ModelLoad(_))));
    }

    #[test]
    #[ignore] // Requires OXIDOC_TEST_MODEL pointing to a GGUF embedding model.
    fn build_vectors_with_model() {
        let model_path = std::env::var("OXIDOC_TEST_MODEL")
            .expect("Set OXIDOC_TEST_MODEL to a GGUF embedding model path");
        let model_bytes = std::fs::read(&model_path).unwrap();
        let records = vec![
            record(
                "getting-started",
                "Install the CLI tool and create your first project",
            ),
            record(
                "api-reference",
                "The REST API provides endpoints for user management",
            ),
        ];

        let index = build_vectors(&records, EmbeddingModel::Bytes(&model_bytes)).unwrap();
        assert_eq!(index.documents.len(), 2);
        assert_eq!(index.vectors.len(), 2);
        assert!(index.dimension > 0);
        for vector in &index.vectors {
            assert_eq!(vector.len(), index.dimension);
        }
    }

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm(a) * norm(b))
    }

    #[test]
    #[ignore] // Requires OXIDOC_TEST_MODEL pointing to a GGUF embedding model.
    fn related_texts_score_above_unrelated() {
        let model_path = std::env::var("OXIDOC_TEST_MODEL")
            .expect("Set OXIDOC_TEST_MODEL to a GGUF embedding model path");
        let records = vec![
            record("ml", "Rust machine learning framework"),
            record("dl", "deep learning library for Rust"),
            record("pie", "a recipe for apple pie"),
        ];

        let index = build_vectors(&records, EmbeddingModel::File(Path::new(&model_path))).unwrap();
        let v = &index.vectors;
        let related = cosine(&v[0], &v[1]);
        let unrelated = [cosine(&v[0], &v[2]), cosine(&v[1], &v[2])];
        println!(
            "ml~dl {related:.4}  ml~pie {:.4}  dl~pie {:.4}",
            unrelated[0], unrelated[1]
        );
        for score in unrelated {
            assert!(
                related > score + 0.1,
                "related pair {related} is not clearly above unrelated pair {score}"
            );
        }
    }
}
