#[cfg(feature = "wasm")]
use oxidoc_island::IslandError;
use std::path::PathBuf;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

/// Errors from building, loading or querying a search index.
#[derive(Debug)]
pub enum SearchError {
    /// The embedding model failed to open, parse or load.
    ModelLoad(String),
    /// An index file failed to decode.
    IndexLoad(String),
    /// Computing an embedding failed.
    Embedding(String),
    /// Serialising an in-memory value failed.
    Serialization(String),
    /// A browser API call failed.
    Js(String),
    /// The query is malformed.
    InvalidQuery(String),
    /// A tensor operation failed.
    Numr(String),
    /// Encoding the index file at `path` failed.
    Encode {
        /// File the encoded bytes were meant for.
        path: PathBuf,
        /// Encoder error text.
        message: String,
    },
    /// An index file has a format version this reader does not support.
    UnsupportedVersion {
        /// Name of the index file.
        file: String,
        /// Version stored in the file.
        found: u32,
        /// Version this reader supports.
        supported: u32,
    },
    /// Writing the index file at `path` failed.
    Write {
        /// File that failed to write.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::ModelLoad(msg) => write!(f, "Model load error: {}", msg),
            SearchError::IndexLoad(msg) => write!(f, "Index load error: {}", msg),
            SearchError::Embedding(msg) => write!(f, "Embedding error: {}", msg),
            SearchError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            SearchError::Js(msg) => write!(f, "JS interop error: {}", msg),
            SearchError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            SearchError::Numr(msg) => write!(f, "Numr error: {}", msg),
            SearchError::Encode { path, message } => {
                write!(f, "Failed to encode {}: {}", path.display(), message)
            }
            SearchError::UnsupportedVersion {
                file,
                found,
                supported,
            } => write!(
                f,
                "{file} has format version {found}, but this reader supports version {supported}; \
                 rebuild the index with a matching oxidoc-search"
            ),
            SearchError::Write { path, source } => {
                write!(f, "Failed to write {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SearchError::Write { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(feature = "wasm")]
impl From<JsValue> for SearchError {
    fn from(e: JsValue) -> Self {
        SearchError::Js(format!("{:?}", e))
    }
}

impl From<serde_json::Error> for SearchError {
    fn from(e: serde_json::Error) -> Self {
        SearchError::Serialization(e.to_string())
    }
}

#[cfg(feature = "wasm")]
impl From<SearchError> for IslandError {
    fn from(e: SearchError) -> Self {
        IslandError {
            message: e.to_string(),
        }
    }
}

/// Result alias for search operations.
pub type SearchResult<T> = Result<T, SearchError>;
