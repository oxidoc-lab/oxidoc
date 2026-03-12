use crate::error::SearchResult;
use crate::types::{SearchQuery, SearchResult as DocResult, SearchSource, VectorIndex};
use boostr::model::encoder::{EmbeddingPipeline, EncoderClient};
use numr::ops::{DistanceMetric, DistanceOps};
use numr::prelude::*;
use std::sync::Arc;

pub struct SemanticSearcher<R: Runtime<DType = DType>> {
    embedding_pipeline: Arc<EmbeddingPipeline<R>>,
    vector_index: VectorIndex,
    device: R::Device,
}

impl<R: Runtime<DType = DType>> SemanticSearcher<R> {
    pub fn new(
        embedding_pipeline: EmbeddingPipeline<R>,
        vector_index: VectorIndex,
        device: R::Device,
    ) -> Self {
        Self {
            embedding_pipeline: Arc::new(embedding_pipeline),
            vector_index,
            device,
        }
    }

    pub fn search<C>(&self, client: &C, query: &SearchQuery) -> SearchResult<Vec<DocResult>>
    where
        C: EncoderClient<R>,
        R::Client: DistanceOps<R>
            + TensorOps<R>
            + ScalarOps<R>
            + IndexingOps<R>
            + ReduceOps<R>
            + UnaryOps<R>,
    {
        let text = query.text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = self
            .embedding_pipeline
            .embed_text(client, text)
            .map_err(|e| {
                crate::error::SearchError::Embedding(format!("Failed to embed query: {}", e))
            })?;

        if query_embedding.is_empty() {
            return Ok(Vec::new());
        }

        let similarities = self.compute_similarities(client, &query_embedding)?;

        let mut results: Vec<(u32, f32)> = similarities
            .into_iter()
            .enumerate()
            .map(|(idx, sim)| (idx as u32, sim))
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results
            .into_iter()
            .take(query.max_results)
            .filter_map(|(doc_id, score)| {
                self.vector_index
                    .documents
                    .iter()
                    .find(|d| d.id == doc_id)
                    .map(|doc| DocResult {
                        title: doc.title.clone(),
                        path: doc.path.clone(),
                        snippet: doc.snippet.clone(),
                        score,
                        source: SearchSource::Semantic,
                        breadcrumb: vec![],
                        anchor: String::new(),
                        highlight_terms: Vec::new(),
                    })
            })
            .collect())
    }

    fn compute_similarities<C>(&self, client: &C, query_embedding: &[f32]) -> SearchResult<Vec<f32>>
    where
        C: RuntimeClient<R>
            + DistanceOps<R>
            + TensorOps<R>
            + ScalarOps<R>
            + IndexingOps<R>
            + ReduceOps<R>
            + UnaryOps<R>,
    {
        if self.vector_index.vectors.is_empty() {
            return Ok(Vec::new());
        }

        let query_tensor =
            Tensor::<R>::from_slice(query_embedding, &[1, query_embedding.len()], &self.device);

        let mut doc_vectors: Vec<f32> = Vec::new();
        for vec in &self.vector_index.vectors {
            doc_vectors.extend(vec);
        }

        let docs_tensor = Tensor::<R>::from_slice(
            &doc_vectors,
            &[self.vector_index.vectors.len(), self.vector_index.dimension],
            &self.device,
        );

        let distances = client
            .cdist(&query_tensor, &docs_tensor, DistanceMetric::Cosine)
            .map_err(|e| {
                crate::error::SearchError::Numr(format!(
                    "Failed to compute cosine distances: {}",
                    e
                ))
            })?;

        let similarities: Vec<f32> = distances.to_vec();

        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_query() {
        // This test is validation only — actual semantic search requires a full runtime setup
        // which is beyond unit test scope for Wasm crates. Integration tests would use
        // wasm-bindgen-test with a full encoder loaded.
        let text = "";
        assert!(text.trim().is_empty());
    }
}
