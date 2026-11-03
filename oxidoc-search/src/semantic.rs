//! Embedding searcher over an int8 vector store.
//!
//! The searcher holds vectors and doc ids only. The caller maps doc ids to
//! document metadata, so document text is stored once, in the lexical index.

use crate::embed_input::embed_one;
use crate::error::{SearchError, SearchResult};
use crate::types::SearchQuery;
use crate::vectors::QuantVectors;
use boostr::DequantOps;
use boostr::model::encoder::{EmbeddingPipeline, EncoderClient};
use numr::prelude::*;

/// One semantic hit: a doc id and its cosine similarity to the query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticHit {
    /// Doc id, as in `search-meta.bin`.
    pub doc_id: u32,
    /// Cosine similarity to the query. Higher ranks first.
    pub score: f32,
}

/// Embedding searcher over precomputed int8 document vectors.
pub struct SemanticSearcher<R: Runtime<DType = DType>> {
    embedding_pipeline: EmbeddingPipeline<R>,
    vectors: QuantVectors,
}

impl<R: Runtime<DType = DType>> SemanticSearcher<R> {
    /// Creates a searcher from an embedding pipeline and the stored document vectors.
    pub fn new(embedding_pipeline: EmbeddingPipeline<R>, vectors: QuantVectors) -> Self {
        Self {
            embedding_pipeline,
            vectors,
        }
    }

    /// Stored document vectors.
    pub fn vectors(&self) -> &QuantVectors {
        &self.vectors
    }

    /// Ranks every document by cosine similarity to the query embedding.
    ///
    /// Returns the best `query.max_results` hits. `query.filters` is ignored here.
    pub fn search<C>(&self, client: &C, query: &SearchQuery) -> SearchResult<Vec<SemanticHit>>
    where
        C: EncoderClient<R>,
        R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R> + DequantOps<R>,
    {
        self.search_allowed(client, query, &|_| true)
    }

    /// Ranks the documents for which `allow` returns true.
    ///
    /// Returns the best `query.max_results` of them. Ties keep doc id order.
    /// A document with a zero vector never ranks.
    pub(crate) fn search_allowed<C>(
        &self,
        client: &C,
        query: &SearchQuery,
        allow: &impl Fn(u32) -> bool,
    ) -> SearchResult<Vec<SemanticHit>>
    where
        C: EncoderClient<R>,
        R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R> + DequantOps<R>,
    {
        let text = query.text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let query_embedding = embed_one(&self.embedding_pipeline, client, text)
            .map_err(|e| SearchError::Embedding(format!("Failed to embed query: {}", e)))?;

        let mut hits: Vec<SemanticHit> = self
            .vectors
            .cosine_scores(&query_embedding)?
            .into_iter()
            .filter(|&(doc_id, _)| allow(doc_id))
            .map(|(doc_id, score)| SemanticHit { doc_id, score })
            .collect();
        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        hits.truncate(query.max_results);
        Ok(hits)
    }
}
