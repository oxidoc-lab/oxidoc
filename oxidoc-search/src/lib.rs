pub mod engine;
pub mod error;
pub mod index;
pub mod island;
pub mod lexical;
pub mod semantic;
pub mod types;
pub mod wasm;

pub use engine::SearchEngine;
pub use error::{SearchError, SearchResult};
pub use island::SemanticSearch;
pub use lexical::LexicalSearcher;
pub use semantic::SemanticSearcher;
pub use types::{
    ChunkEntry, ChunkManifest, ChunkPostings, DocMetadata, LexicalIndex, Posting, SearchMetadata,
    SearchQuery, SearchResult as SearchDoc, SearchSource, VectorIndex,
};
