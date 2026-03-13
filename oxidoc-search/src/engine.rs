use crate::error::SearchResult;
use crate::lexical::LexicalSearcher;
use crate::semantic::SemanticSearcher;
use crate::types::{SearchQuery, SearchResult as DocResult};
use boostr::model::encoder::EncoderClient;
use numr::prelude::*;
use splintr::Tokenize;
use std::collections::HashMap;

pub struct SearchEngine<R: Runtime<DType = DType>, T: Tokenize> {
    semantic: Option<SemanticSearcher<R, T>>,
    lexical: LexicalSearcher,
}

impl<R: Runtime<DType = DType>, T: Tokenize> SearchEngine<R, T> {
    pub fn new(lexical: LexicalSearcher, semantic: Option<SemanticSearcher<R, T>>) -> Self {
        Self { semantic, lexical }
    }

    /// Hybrid search: lexical + semantic with RRF fusion.
    /// Falls back to lexical-only if no semantic searcher is configured.
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

        // Weighted RRF: lexical matches are more important than semantic
        const LEXICAL_WEIGHT: f32 = 0.7;
        const SEMANTIC_WEIGHT: f32 = 0.3;

        let mut all_results: HashMap<String, (DocResult, f32)> = HashMap::new();

        // Semantic results (if available)
        if let Some(ref semantic) = self.semantic {
            let semantic_results = semantic.search(client, query)?;
            for (rank, result) in semantic_results.iter().enumerate() {
                let score = rrf_score(rank as u32) * SEMANTIC_WEIGHT;
                all_results
                    .entry(result.path.clone())
                    .and_modify(|(_, s)| *s += score)
                    .or_insert_with(|| (result.clone(), score));
            }
        }

        // Lexical results (always, higher weight)
        let lexical_results = self.lexical.search(query);
        for (rank, result) in lexical_results.iter().enumerate() {
            let score = rrf_score(rank as u32) * LEXICAL_WEIGHT;
            all_results
                .entry(result.path.clone())
                .and_modify(|(_, s)| *s += score)
                .or_insert_with(|| (result.clone(), score));
        }

        let mut fused_results: Vec<(DocResult, f32)> = all_results.into_values().collect();

        fused_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(fused_results
            .into_iter()
            .take(query.max_results)
            .map(|(mut result, score)| {
                result.score = score;
                result
            })
            .collect())
    }

    /// Attach a semantic searcher (enables hybrid search).
    pub fn set_semantic(&mut self, semantic: SemanticSearcher<R, T>) {
        self.semantic = Some(semantic);
    }

    /// Whether semantic search is available.
    pub fn has_semantic(&self) -> bool {
        self.semantic.is_some()
    }

    /// Lexical-only search (no model needed).
    pub fn search_lexical(&self, query: &SearchQuery) -> Vec<DocResult> {
        self.lexical.search(query)
    }

    /// Load a chunk into the lexical searcher.
    pub fn load_chunk(&mut self, data: &[u8]) -> SearchResult<()> {
        self.lexical.load_chunk(data)
    }

    /// Get chunk IDs needed for a query.
    pub fn needed_chunk_ids(&self, query: &str) -> Vec<u32> {
        self.lexical.needed_chunk_ids(query)
    }
}

/// Reciprocal Rank Fusion score.
fn rrf_score(rank: u32) -> f32 {
    1.0 / (rank as f32 + 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_score_decreases() {
        let score0 = rrf_score(0);
        let score1 = rrf_score(1);
        let score10 = rrf_score(10);

        assert!(score0 > score1);
        assert!(score1 > score10);
    }

    #[test]
    fn test_rrf_score_positive() {
        for rank in 0..100 {
            let score = rrf_score(rank);
            assert!(score > 0.0);
            assert!(score <= 1.0 / 60.0);
        }
    }

    #[test]
    fn test_rrf_fusion_order() {
        let score_0 = rrf_score(0);
        let score_1 = rrf_score(1);

        let avg = (score_0 + score_1) / 2.0;
        assert!(avg > 0.0);
    }
}
