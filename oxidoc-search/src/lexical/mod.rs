mod facets;
mod levenshtein;
mod match_index;
mod matching;
mod query_scorer;
pub(crate) mod rank;
mod resolve;
mod scoring;
mod search;
mod section_hits;
mod vocab;

pub use search::LexicalSearcher;
