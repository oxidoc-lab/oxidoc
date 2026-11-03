#[cfg(feature = "builder")]
pub mod builder;
#[cfg(feature = "semantic")]
pub mod engine;
pub mod error;
pub mod index;
#[cfg(feature = "wasm")]
pub mod island;
pub mod lexical;
pub mod record;
#[cfg(feature = "semantic")]
pub mod semantic;
pub mod types;
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "semantic")]
pub use engine::SearchEngine;
pub use error::{SearchError, SearchResult};
#[cfg(feature = "wasm")]
pub use island::SemanticSearch;
pub use lexical::LexicalSearcher;
pub use record::{Field, Record};
#[cfg(feature = "semantic")]
pub use semantic::SemanticSearcher;
pub use types::{
    ChunkEntry, ChunkManifest, ChunkPostings, DocMetadata, HeadingPos, LexicalIndex, Posting,
    SearchMetadata, SearchQuery, SearchResult as SearchDoc, SearchSource, VectorIndex,
};
