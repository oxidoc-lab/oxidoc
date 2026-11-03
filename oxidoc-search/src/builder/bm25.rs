//! BM25 term scoring.

/// BM25 term-frequency saturation.
pub const K1: f32 = 1.2;
/// BM25 document-length normalisation.
pub const B: f32 = 0.75;

/// Returns the BM25 inverse document frequency of a term found in `df` of `total_docs` docs.
pub fn compute_idf(df: u32, total_docs: u32) -> f32 {
    let df = (df as f32).max(1.0);
    ((total_docs as f32 - df + 0.5) / (df + 0.5) + 1.0).ln()
}

/// Returns the BM25 score of a term with frequency `tf` in a doc of `doc_length` tokens.
pub fn compute_bm25(idf: f32, tf: f32, doc_length: f32, avg_doc_length: f32) -> f32 {
    let normalizer = 1.0 - B + B * (doc_length / avg_doc_length);
    idf * ((tf * (K1 + 1.0)) / (tf + K1 * normalizer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rare_terms_have_higher_idf() {
        assert!(compute_idf(1, 100) > compute_idf(50, 100));
    }

    #[test]
    fn bm25_grows_with_term_frequency() {
        let idf = compute_idf(1, 10);
        assert!(compute_bm25(idf, 3.0, 10.0, 10.0) > compute_bm25(idf, 1.0, 10.0, 10.0));
    }
}
