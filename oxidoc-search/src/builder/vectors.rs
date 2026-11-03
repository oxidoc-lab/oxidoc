//! Semantic vector index built from records with a GGUF embedding model.
//!
//! The builder embeds every record body on the CPU and stores each vector as
//! int8 in `search-vectors.bin`. The format is in [`crate::vectors::codec`]. The
//! browser loads the same model, embeds only the query, and ranks the stored
//! vectors by cosine similarity.

use std::path::{Path, PathBuf};

use boostr::format::gguf::Gguf;
use boostr::model::encoder::EmbeddingPipeline;
use numr::runtime::cpu::{CpuClient, CpuDevice, CpuRuntime};

use crate::embed_input::embed_each;
use crate::error::SearchError;
use crate::record::Record;
use crate::vectors::{QuantVectors, encode_vectors};

pub use crate::vectors::VECTORS_FILE;
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

/// Embeds each record body and returns the int8 vector index.
///
/// Each body is embedded alone, cut to the model's maximum sequence length.
/// A body with no tokens gets a zero vector, and semantic search never returns it.
/// Doc ids follow input order, as in the lexical index. Fails when an embedding
/// holds NaN or infinity.
pub fn build_vectors(
    records: &[Record],
    model: EmbeddingModel<'_>,
) -> Result<QuantVectors, SearchError> {
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

    let pipeline = EmbeddingPipeline::<CpuRuntime>::from_gguf(&mut gguf, device)
        .map_err(|e| SearchError::ModelLoad(format!("Failed to load embedding model: {}", e)))?;
    let dimension = pipeline.config().hidden_size;

    let bodies: Vec<String> = records.iter().map(Record::body).collect();
    let texts: Vec<&str> = bodies.iter().map(String::as_str).collect();
    let embeddings = embed_each(&pipeline, &client, &texts)
        .map_err(|e| SearchError::Embedding(format!("Failed to compute embeddings: {}", e)))?;

    let mut vectors = QuantVectors::with_capacity(dimension, embeddings.len());
    for (index, (embedding, record)) in embeddings.iter().zip(records).enumerate() {
        let doc_id = u32::try_from(index)
            .map_err(|_| SearchError::Embedding(format!("more than {} records", u32::MAX)))?;
        vectors.push(doc_id, embedding).map_err(|e| {
            SearchError::Embedding(format!(
                "embedding of record {index} ('{}'): {e}",
                record.id
            ))
        })?;
    }
    Ok(vectors)
}

/// Writes `vectors` to `search-vectors.bin` in `dir`. `dir` must exist.
pub fn write_vectors(vectors: &QuantVectors, dir: &Path) -> Result<(), SearchError> {
    let path = dir.join(VECTORS_FILE);
    let bytes = encode_vectors(vectors).map_err(|e| SearchError::Encode {
        path: path.clone(),
        message: e.to_string(),
    })?;
    std::fs::write(&path, bytes).map_err(|source| SearchError::Write { path, source })
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
        let mut vectors = QuantVectors::new(2);
        vectors.push(0, &[0.5, 0.25]).unwrap();
        write_vectors(&vectors, tmp.path()).unwrap();
        let bytes = std::fs::read(tmp.path().join(VECTORS_FILE)).unwrap();
        assert!(bytes.starts_with(b"OXVEC"));
        let back = crate::vectors::decode_vectors(&bytes).unwrap();
        assert_eq!(back, vectors);
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

        let vectors = build_vectors(&records, EmbeddingModel::Bytes(&model_bytes)).unwrap();
        assert_eq!(vectors.len(), 2);
        assert_eq!(vectors.doc_ids(), &[0, 1]);
        assert!(vectors.dimension() > 0);
        for index in 0..vectors.len() {
            assert_eq!(vectors.values(index).unwrap().len(), vectors.dimension());
        }
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

        let built = build_vectors(&records, EmbeddingModel::File(Path::new(&model_path))).unwrap();
        // Score through the stored file: encode, decode, then int8 cosine.
        let vectors = crate::vectors::decode_vectors(&encode_vectors(&built).unwrap()).unwrap();
        let cosine = |a: usize, b: usize| {
            let query = vectors.dequantize(a).unwrap();
            let scores = vectors.cosine_scores(&query).unwrap();
            scores
                .iter()
                .find(|(doc_id, _)| *doc_id as usize == b)
                .map(|(_, score)| *score)
                .unwrap()
        };
        let related = cosine(0, 1);
        let unrelated = [cosine(0, 2), cosine(1, 2)];
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
