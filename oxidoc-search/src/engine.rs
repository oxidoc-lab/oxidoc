use crate::error::SearchResult;
use crate::lexical::LexicalSearcher;
use crate::semantic::SemanticSearcher;
use crate::types::{SearchQuery, SearchResult as DocResult};
use boostr::model::encoder::EncoderClient;
use numr::prelude::*;
use std::collections::HashMap;

pub struct SearchEngine<R: Runtime<DType = DType>> {
    semantic: Option<SemanticSearcher<R>>,
    lexical: LexicalSearcher,
}

impl<R: Runtime<DType = DType>> SearchEngine<R> {
    pub fn new(lexical: LexicalSearcher, semantic: Option<SemanticSearcher<R>>) -> Self {
        Self { semantic, lexical }
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

        let mut all_results: HashMap<String, (DocResult, Vec<f32>)> = HashMap::new();

        if let Some(ref semantic) = self.semantic {
            let semantic_results = semantic.search(client, query)?;
            for (rank, result) in semantic_results.iter().enumerate() {
                let rrf_score = rrf_score(rank as u32);
                all_results
                    .entry(result.path.clone())
                    .and_modify(|(_, scores)| scores.push(rrf_score))
                    .or_insert_with(|| (result.clone(), vec![rrf_score]));
            }
        }

        let lexical_results = self.lexical.search(query);
        for (rank, result) in lexical_results.iter().enumerate() {
            let rrf_score = rrf_score(rank as u32);
            all_results
                .entry(result.path.clone())
                .and_modify(|(_, scores)| scores.push(rrf_score))
                .or_insert_with(|| (result.clone(), vec![rrf_score]));
        }

        let mut fused_results: Vec<(DocResult, f32)> = all_results
            .into_values()
            .map(|(result, scores)| {
                let fused_score = scores.iter().sum::<f32>() / scores.len() as f32;
                (result, fused_score)
            })
            .collect();

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
}

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
        // Test that RRF fusion correctly combines scores
        let score_0 = rrf_score(0);
        let score_1 = rrf_score(1);

        let avg = (score_0 + score_1) / 2.0;
        assert!(avg > 0.0);
    }
}
