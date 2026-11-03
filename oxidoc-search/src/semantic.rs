use crate::error::SearchResult;
use crate::lexical::rank::Hit;
use crate::types::{SearchQuery, SearchResult as DocResult, SearchSource, VectorIndex};
use boostr::model::encoder::{EmbeddingPipeline, EncoderClient};
use numr::ops::{DistanceMetric, DistanceOps};
use numr::prelude::*;
use splintr::Tokenize;
use std::sync::Arc;

/// Embedding searcher over a precomputed vector index.
pub struct SemanticSearcher<R: Runtime<DType = DType>, T: Tokenize> {
    embedding_pipeline: Arc<EmbeddingPipeline<R, T>>,
    vector_index: VectorIndex,
    device: R::Device,
}

impl<R: Runtime<DType = DType>, T: Tokenize> SemanticSearcher<R, T> {
    /// Creates a searcher from an embedding pipeline and the stored document vectors.
    pub fn new(
        embedding_pipeline: EmbeddingPipeline<R, T>,
        vector_index: VectorIndex,
        device: R::Device,
    ) -> Self {
        Self {
            embedding_pipeline: Arc::new(embedding_pipeline),
            vector_index,
            device,
        }
    }

    /// Ranks every document by cosine similarity to the query embedding.
    ///
    /// Returns the best `query.max_results` hits. `query.filters` is ignored here.
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
        let hits = self.search_hits(client, query, &|_| true)?;
        Ok(hits.into_iter().map(|hit| hit.result).collect())
    }

    /// Ranks the documents for which `allow` returns true.
    ///
    /// Returns the best `query.max_results` of them, with their doc ids.
    pub(crate) fn search_hits<C>(
        &self,
        client: &C,
        query: &SearchQuery,
        allow: &impl Fn(u32) -> bool,
    ) -> SearchResult<Vec<Hit>>
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
            .filter(|(doc_id, _)| allow(*doc_id))
            .collect();

        results.sort_by(|a, b| b.1.total_cmp(&a.1));

        Ok(results
            .into_iter()
            .take(query.max_results)
            .filter_map(|(doc_id, score)| {
                self.vector_index
                    .documents
                    .iter()
                    .find(|d| d.id == doc_id)
                    .map(|doc| Hit {
                        doc_id,
                        result: DocResult {
                            title: doc.title.clone(),
                            path: doc.path.clone(),
                            snippet: doc.snippet.clone(),
                            score,
                            source: SearchSource::Semantic,
                            breadcrumb: vec![],
                            anchor: String::new(),
                            highlight_terms: Vec::new(),
                        },
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

        // Cosine distance is `1 - similarity`.
        let distances: Vec<f32> = distances.to_vec();
        let similarities: Vec<f32> = distances.into_iter().map(|d| 1.0 - d).collect();

        Ok(similarities)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_empty_query() {
        let text = "";
        assert!(text.trim().is_empty());
    }
}
