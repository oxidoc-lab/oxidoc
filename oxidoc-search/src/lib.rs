#[cfg(feature = "builder")]
pub mod builder;
#[cfg(feature = "semantic")]
pub mod embed_input;
#[cfg(feature = "semantic")]
pub mod engine;
pub mod error;
pub mod index;
#[cfg(feature = "wasm")]
pub mod island;
pub mod lexical;
pub mod record;
pub mod records;
#[cfg(feature = "semantic")]
pub mod semantic;
pub mod types;
#[cfg(feature = "semantic")]
pub mod vectors;
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(feature = "semantic")]
pub use engine::SearchEngine;
pub use error::{SearchError, SearchResult};
#[cfg(feature = "wasm")]
pub use island::SemanticSearch;
pub use lexical::LexicalSearcher;
pub use record::{Field, Record};
pub use records::{DocRecord, Facets, FieldRange, Filter, SearchRecords};
#[cfg(feature = "semantic")]
pub use semantic::{SemanticHit, SemanticSearcher};
pub use types::{
    ChunkEntry, ChunkManifest, ChunkPostings, DocMetadata, HeadingPos, LexicalIndex, Posting,
    SearchMetadata, SearchQuery, SearchResult as SearchDoc, SearchSource,
};
#[cfg(feature = "semantic")]
pub use vectors::QuantVectors;
