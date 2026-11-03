//! In-memory int8 vector store and its cosine ranking.
//!
//! Each document vector holds int8 values and one f32 scale. The query embedding
//! stays f32. A document scores `dot(q, query) / (|q| * |query|)`, where `q` is
//! its int8 values and `|q|` is precomputed at load.
//!
//! The scale cancels out of the cosine, so the ranking never reads it. Keeping the
//! query in f32 means only the document side carries rounding error. Quantising
//! the query too would add a second error of the same size. The f32 dot costs
//! one multiply-add per element, the same count as an i32 dot. Both are small
//! next to the encoder forward that embeds the query.

use crate::error::{SearchError, SearchResult};

use super::quantize::quantize_into;

/// Why [`QuantVectors::push`] refused a vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushError {
    /// The vector length is not the store dimension.
    WrongLength {
        /// Length of the vector.
        found: usize,
        /// Dimension of the store.
        expected: usize,
    },
    /// The vector holds NaN or infinity.
    NonFinite,
    /// The doc id is not above the last stored doc id.
    DocIdOrder {
        /// Doc id of the vector.
        doc_id: u32,
        /// Last stored doc id.
        last: u32,
    },
}

impl std::fmt::Display for PushError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PushError::WrongLength { found, expected } => {
                write!(f, "vector has {found} values, expected {expected}")
            }
            PushError::NonFinite => write!(f, "vector holds NaN or infinity"),
            PushError::DocIdOrder { doc_id, last } => {
                write!(f, "doc id {doc_id} is not above the previous doc id {last}")
            }
        }
    }
}

/// Int8 document vectors with strictly increasing doc ids.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantVectors {
    dimension: usize,
    doc_ids: Vec<u32>,
    scales: Vec<f32>,
    /// `doc_ids.len() * dimension` values, one row per vector.
    values: Vec<i8>,
    /// Euclidean norm of each row of `values`.
    norms: Vec<f32>,
}

/// Returns the Euclidean norm of int8 `values`.
fn norm_of(values: &[i8]) -> f32 {
    let sum: u64 = values
        .iter()
        .map(|&v| u64::from(v.unsigned_abs()).pow(2))
        .sum();
    (sum as f64).sqrt() as f32
}

impl QuantVectors {
    /// Creates an empty store for vectors of length `dimension`.
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            doc_ids: Vec::new(),
            scales: Vec::new(),
            values: Vec::new(),
            norms: Vec::new(),
        }
    }

    /// Creates an empty store with room for `count` vectors.
    pub(crate) fn with_capacity(dimension: usize, count: usize) -> Self {
        Self {
            dimension,
            doc_ids: Vec::with_capacity(count),
            scales: Vec::with_capacity(count),
            values: Vec::with_capacity(count.saturating_mul(dimension)),
            norms: Vec::with_capacity(count),
        }
    }

    /// Quantises `vector` and stores it under `doc_id`.
    ///
    /// `doc_id` must be above the last stored doc id.
    pub fn push(&mut self, doc_id: u32, vector: &[f32]) -> Result<(), PushError> {
        if vector.len() != self.dimension {
            return Err(PushError::WrongLength {
                found: vector.len(),
                expected: self.dimension,
            });
        }
        self.check_order(doc_id)?;
        let start = self.values.len();
        let scale = quantize_into(vector, &mut self.values).ok_or(PushError::NonFinite)?;
        self.finish_row(doc_id, scale, start);
        Ok(())
    }

    /// Stores already quantised `values` under `doc_id`.
    ///
    /// The caller checks that `values` has `dimension` entries in `[-127, 127]`,
    /// that `scale` is finite and not negative, and that `doc_id` is in order.
    pub(super) fn push_raw(&mut self, doc_id: u32, scale: f32, values: &[i8]) {
        let start = self.values.len();
        self.values.extend_from_slice(values);
        self.finish_row(doc_id, scale, start);
    }

    /// Returns an error when `doc_id` is not above the last stored doc id.
    pub(super) fn check_order(&self, doc_id: u32) -> Result<(), PushError> {
        match self.doc_ids.last() {
            Some(&last) if doc_id <= last => Err(PushError::DocIdOrder { doc_id, last }),
            _ => Ok(()),
        }
    }

    /// Records the row that starts at `start` in `values`.
    fn finish_row(&mut self, doc_id: u32, scale: f32, start: usize) {
        self.doc_ids.push(doc_id);
        self.scales.push(scale);
        self.norms.push(norm_of(&self.values[start..]));
    }

    /// Length of every vector.
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Number of stored vectors.
    pub fn len(&self) -> usize {
        self.doc_ids.len()
    }

    /// True when the store holds no vectors.
    pub fn is_empty(&self) -> bool {
        self.doc_ids.is_empty()
    }

    /// Doc ids in storage order, strictly increasing.
    pub fn doc_ids(&self) -> &[u32] {
        &self.doc_ids
    }

    /// Scale of each vector, in storage order.
    pub fn scales(&self) -> &[f32] {
        &self.scales
    }

    /// Int8 values of the vector at storage position `index`.
    pub fn values(&self, index: usize) -> Option<&[i8]> {
        let start = index.checked_mul(self.dimension)?;
        self.values.get(start..start.checked_add(self.dimension)?)
    }

    /// Returns `q * scale` for the vector at storage position `index`.
    pub fn dequantize(&self, index: usize) -> Option<Vec<f32>> {
        let scale = *self.scales.get(index)?;
        let values = self.values(index)?;
        Some(values.iter().map(|&v| f32::from(v) * scale).collect())
    }

    /// Returns `(doc_id, cosine)` for every non-zero vector, in storage order.
    ///
    /// A zero vector belongs to a document with no tokens. It is skipped, since
    /// its cosine is undefined. A zero `query` scores nothing.
    /// Fails when `query` length is not the store dimension.
    pub fn cosine_scores(&self, query: &[f32]) -> SearchResult<Vec<(u32, f32)>> {
        if query.len() != self.dimension {
            return Err(SearchError::InvalidQuery(format!(
                "query embedding has {} values, but the stored vectors have {}; \
                 rebuild the vectors with the model the browser loads",
                query.len(),
                self.dimension
            )));
        }
        let query_norm = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        if query_norm == 0.0 || self.dimension == 0 {
            return Ok(Vec::new());
        }
        let rows = self.values.chunks_exact(self.dimension);
        Ok(self
            .doc_ids
            .iter()
            .zip(&self.norms)
            .zip(rows)
            .filter(|((_, norm), _)| **norm > 0.0)
            .map(|((&doc_id, &norm), row)| {
                let dot: f32 = row.iter().zip(query).map(|(&q, &x)| f32::from(q) * x).sum();
                (doc_id, dot / (norm * query_norm))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vectors::seeded::seeded_normal;
    use std::collections::HashSet;

    fn cosine(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm = |v: &[f32]| v.iter().map(|x| x * x).sum::<f32>().sqrt();
        dot / (norm(a) * norm(b))
    }

    #[test]
    fn push_rejects_bad_vectors() {
        let mut store = QuantVectors::new(2);
        assert_eq!(
            store.push(0, &[1.0]),
            Err(PushError::WrongLength {
                found: 1,
                expected: 2
            })
        );
        assert_eq!(store.push(0, &[1.0, f32::NAN]), Err(PushError::NonFinite));
        store.push(3, &[1.0, 0.0]).unwrap();
        assert_eq!(
            store.push(3, &[1.0, 0.0]),
            Err(PushError::DocIdOrder { doc_id: 3, last: 3 })
        );
        assert_eq!(store.len(), 1);
        assert_eq!(store.values(0), Some(&[127i8, 0][..]));
    }

    #[test]
    fn zero_vectors_and_zero_query_score_nothing() {
        let mut store = QuantVectors::new(2);
        store.push(0, &[0.0, 0.0]).unwrap();
        store.push(1, &[0.5, 0.5]).unwrap();
        let scores = store.cosine_scores(&[1.0, 1.0]).unwrap();
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].0, 1);
        assert!((scores[0].1 - 1.0).abs() < 1e-6);
        assert!(store.cosine_scores(&[0.0, 0.0]).unwrap().is_empty());
    }

    #[test]
    fn query_dimension_mismatch_is_an_error() {
        let store = QuantVectors::new(3);
        assert!(matches!(
            store.cosine_scores(&[1.0]),
            Err(SearchError::InvalidQuery(_))
        ));
    }

    #[test]
    fn int8_cosine_tracks_f32_cosine() {
        let dimension = 384;
        let mut store = QuantVectors::new(dimension);
        let docs: Vec<Vec<f32>> = (0..50).map(|i| seeded_normal(i, dimension)).collect();
        for (i, doc) in docs.iter().enumerate() {
            store.push(i as u32, doc).unwrap();
        }
        let query = seeded_normal(999, dimension);
        for (doc_id, score) in store.cosine_scores(&query).unwrap() {
            let exact = cosine(&docs[doc_id as usize], &query);
            assert!((score - exact).abs() < 2e-3, "{score} vs {exact}");
        }
    }

    /// Ids of the `k` best scores, best first.
    fn top_k(mut scores: Vec<(u32, f32)>, k: usize) -> Vec<u32> {
        scores.sort_by(|a, b| b.1.total_cmp(&a.1));
        scores.into_iter().take(k).map(|(id, _)| id).collect()
    }

    /// Int8 cosine with the query quantised too and an i32 dot product.
    fn int8_int8_scores(store: &QuantVectors, query: &[f32]) -> Vec<(u32, f32)> {
        let mut q_query = Vec::new();
        quantize_into(query, &mut q_query).unwrap();
        let query_norm = norm_of(&q_query);
        (0..store.len())
            .map(|i| {
                let row = store.values(i).unwrap();
                let dot: i32 = row
                    .iter()
                    .zip(&q_query)
                    .map(|(&a, &b)| i32::from(a) * i32::from(b))
                    .sum();
                (
                    store.doc_ids()[i],
                    dot as f32 / (store.norms[i] * query_norm),
                )
            })
            .collect()
    }

    #[test]
    fn top_10_recall_against_f32() {
        let (dimension, docs, queries, k) = (384, 1000, 50, 10);
        let vectors: Vec<Vec<f32>> = (0..docs).map(|i| seeded_normal(i, dimension)).collect();
        let mut store = QuantVectors::new(dimension);
        for (i, v) in vectors.iter().enumerate() {
            store.push(i as u32, v).unwrap();
        }

        let (mut hits_f32_query, mut hits_i8_query) = (0usize, 0usize);
        for q in 0..queries {
            let query = seeded_normal(10_000 + q, dimension);
            let exact: Vec<(u32, f32)> = vectors
                .iter()
                .enumerate()
                .map(|(i, v)| (i as u32, cosine(v, &query)))
                .collect();
            let truth: HashSet<u32> = top_k(exact, k).into_iter().collect();
            let mixed = top_k(store.cosine_scores(&query).unwrap(), k);
            let both = top_k(int8_int8_scores(&store, &query), k);
            hits_f32_query += mixed.iter().filter(|id| truth.contains(id)).count();
            hits_i8_query += both.iter().filter(|id| truth.contains(id)).count();
        }
        let recall = |hits: usize| hits as f64 / queries as f64;
        println!(
            "top-{k} overlap per query: f32 query {:.3}, int8 query {:.3}",
            recall(hits_f32_query),
            recall(hits_i8_query)
        );
        assert!(recall(hits_f32_query) >= 9.0);
    }
}
