use oxidoc_island::IslandError;
use serde_json;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum SearchError {
    ModelLoad(String),
    IndexLoad(String),
    Embedding(String),
    Serialization(String),
    Js(String),
    InvalidQuery(String),
    Numr(String),
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
        }
    }
}

impl std::error::Error for SearchError {}

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

impl From<SearchError> for IslandError {
    fn from(e: SearchError) -> Self {
        IslandError {
            message: e.to_string(),
        }
    }
}

pub type SearchResult<T> = Result<T, SearchError>;
