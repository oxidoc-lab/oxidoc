use crate::error::SearchResult;
use crate::lexical::LexicalSearcher;
use crate::lexical::rank::Hit;
use crate::records::{Facets, Filter};
use crate::semantic::{SemanticHit, SemanticSearcher};
use crate::types::{DocMetadata, SearchQuery, SearchResult as DocResult, SearchSource};
use boostr::DequantOps;
use boostr::model::encoder::EncoderClient;
use numr::prelude::*;
use std::collections::HashMap;

/// Weight of a lexical rank in the fused score.
const LEXICAL_WEIGHT: f32 = 0.7;
/// Weight of a semantic rank in the fused score.
const SEMANTIC_WEIGHT: f32 = 0.3;

/// Hybrid searcher: lexical always, semantic once a model is attached.
pub struct SearchEngine<R: Runtime<DType = DType>> {
    semantic: Option<SemanticSearcher<R>>,
    lexical: LexicalSearcher,
}

impl<R: Runtime<DType = DType>> SearchEngine<R> {
    /// Creates an engine from a lexical searcher and an optional semantic searcher.
    pub fn new(lexical: LexicalSearcher, semantic: Option<SemanticSearcher<R>>) -> Self {
        Self { semantic, lexical }
    }

    /// Hybrid search: lexical + semantic with RRF fusion.
    /// Falls back to lexical-only if no semantic searcher is configured.
    ///
    /// `query.filters` limits both rankers before fusion. The document boost
    /// multiplies the fused score.
    pub fn search<C>(&self, client: &C, query: &SearchQuery) -> SearchResult<Vec<DocResult>>
    where
        C: EncoderClient<R>,
        R::Client: TensorOps<R> + ScalarOps<R> + IndexingOps<R> + DequantOps<R>,
    {
        let text = query.text.trim();
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_results: HashMap<String, (Hit, f32)> = HashMap::new();
        let mut add = |hits: Vec<Hit>, weight: f32| {
            for (rank, hit) in hits.into_iter().enumerate() {
                let score = rrf_score(rank as u32) * weight;
                all_results
                    .entry(hit.result.path.clone())
                    .and_modify(|(_, s)| *s += score)
                    .or_insert((hit, score));
            }
        };

        // Semantic results (if available)
        if let Some(ref semantic) = self.semantic {
            let allow = |doc_id: u32| self.lexical.passes_filters(doc_id, &query.filters);
            let hits = semantic
                .search_allowed(client, query, &allow)?
                .into_iter()
                .filter_map(|hit| {
                    let doc = self.lexical.document(hit.doc_id)?;
                    Some(semantic_hit(doc, hit))
                })
                .collect();
            add(hits, SEMANTIC_WEIGHT);
        }

        // Lexical results (always, higher weight)
        let mut lexical_hits = self.lexical.search_hits(&query.text, &query.filters);
        lexical_hits.truncate(query.max_results);
        add(lexical_hits, LEXICAL_WEIGHT);

        let mut fused_results: Vec<(Hit, f32)> = all_results
            .into_values()
            .map(|(hit, score)| {
                let boosted = score * self.lexical.boost_of(hit.doc_id);
                (hit, boosted)
            })
            .collect();

        fused_results.sort_by(|a, b| b.1.total_cmp(&a.1));

        Ok(fused_results
            .into_iter()
            .take(query.max_results)
            .map(|(hit, score)| {
                let mut result = self.lexical.tagged_result(hit);
                result.score = score;
                result
            })
            .collect())
    }

    /// Attach a semantic searcher (enables hybrid search).
    pub fn set_semantic(&mut self, semantic: SemanticSearcher<R>) {
        self.semantic = Some(semantic);
    }

    /// Whether semantic search is available.
    pub fn has_semantic(&self) -> bool {
        self.semantic.is_some()
    }

    /// Returns the number of documents in the lexical index.
    pub fn document_count(&self) -> usize {
        self.lexical.document_count()
    }

    /// Lexical-only search (no model needed).
    pub fn search_lexical(&self, query: &SearchQuery) -> Vec<DocResult> {
        self.lexical.search(query)
    }

    /// Load a chunk into the lexical searcher.
    pub fn load_chunk(&mut self, data: &[u8]) -> SearchResult<()> {
        self.lexical.load_chunk(data)
    }

    /// Loads `search-records.bin` bytes into the lexical searcher.
    pub fn load_records(&mut self, data: &[u8]) -> SearchResult<()> {
        self.lexical.load_records(data)
    }

    /// Facet counts for `query` under `filters`. See [`LexicalSearcher::facets`].
    pub fn facets(&self, query: &str, filters: &[Filter]) -> Facets {
        self.lexical.facets(query, filters)
    }

    /// Get chunk IDs needed for a query.
    pub fn needed_chunk_ids(&self, query: &str) -> Vec<u32> {
        self.lexical.needed_chunk_ids(query)
    }
}

/// Builds the result of a semantic hit from the lexical metadata of its document.
fn semantic_hit(doc: &DocMetadata, hit: SemanticHit) -> Hit {
    Hit {
        doc_id: hit.doc_id,
        result: DocResult {
            title: doc.title.clone(),
            path: doc.path.clone(),
            snippet: doc.snippet.clone(),
            score: hit.score,
            source: SearchSource::Semantic,
            breadcrumb: vec![],
            anchor: String::new(),
            highlight_terms: Vec::new(),
            tags: Vec::new(),
        },
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
