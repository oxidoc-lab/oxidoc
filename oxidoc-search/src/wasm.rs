//! Wasm-bindgen exports for the search dialog.
//!
//! Supports two modes:
//! - **Lexical-only** (default): JS fetches `search-meta.bin` + chunks → fast keyword search
//! - **Hybrid** (if model available): JS also fetches `search-model.gguf` + `search-vectors.json`
//!   → lexical + semantic search fused via RRF
//!
//! Flow:
//! 1. JS fetches `/search-meta.bin` → calls `oxidoc_search_init(data)`
//! 2. Per keystroke: `oxidoc_search_needed_chunks(query)` → fetch chunks → `oxidoc_search_load_chunk(data)`
//! 3. Optionally, with the `semantic` feature: fetch `/search-model.gguf` and
//!    `/search-vectors.json` → `oxidoc_search_load_semantic(model, vectors_json)`
//! 4. `oxidoc_search_query(text, max)` → lexical JSON results
//! 5. `oxidoc_search_query_ai(text, max)` → hybrid JSON results (`semantic` feature only)

#[cfg(feature = "semantic")]
use crate::engine::SearchEngine;
#[cfg(feature = "semantic")]
use crate::index::deserialize_vector_index;
use crate::lexical::LexicalSearcher;
#[cfg(feature = "semantic")]
use crate::semantic::SemanticSearcher;
use crate::types::{SearchQuery, SearchResult as DocResult};
#[cfg(feature = "semantic")]
use boostr::format::Gguf;
#[cfg(feature = "semantic")]
use boostr::{CpuClient, CpuDevice, CpuRuntime, EmbeddingPipeline, GgufTokenizer};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// The hybrid search engine: lexical always, semantic when model is loaded.
#[cfg(feature = "semantic")]
type Engine = SearchEngine<CpuRuntime, GgufTokenizer>;

/// The lexical-only search engine used without the `semantic` feature.
#[cfg(not(feature = "semantic"))]
type Engine = LexicalSearcher;

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

#[cfg(feature = "semantic")]
thread_local! {
    static CLIENT: RefCell<Option<CpuClient>> = const { RefCell::new(None) };
}

/// Wraps the lexical searcher in a hybrid engine with no semantic half.
#[cfg(feature = "semantic")]
fn new_engine(searcher: LexicalSearcher) -> Engine {
    SearchEngine::new(searcher, None)
}

/// Uses the lexical searcher as the engine.
#[cfg(not(feature = "semantic"))]
fn new_engine(searcher: LexicalSearcher) -> Engine {
    searcher
}

/// Runs a lexical-only query on the engine.
#[cfg(feature = "semantic")]
fn lexical_query(engine: &Engine, query: &SearchQuery) -> Vec<DocResult> {
    engine.search_lexical(query)
}

/// Runs a lexical-only query on the engine.
#[cfg(not(feature = "semantic"))]
fn lexical_query(engine: &Engine, query: &SearchQuery) -> Vec<DocResult> {
    engine.search(query)
}

/// Initialize the search engine with the metadata binary (documents + chunk manifest).
/// Called once after fetching `/search-meta.bin`.
#[wasm_bindgen]
pub fn oxidoc_search_init(data: &[u8]) -> Result<(), JsValue> {
    let searcher =
        LexicalSearcher::from_metadata(data).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let engine = new_engine(searcher);

    ENGINE.with(|e| {
        *e.borrow_mut() = Some(engine);
    });

    #[cfg(feature = "semantic")]
    {
        let client = CpuClient::new(CpuDevice::new());
        CLIENT.with(|c| {
            *c.borrow_mut() = Some(client);
        });
    }
    Ok(())
}

/// Load a chunk's postings into the searcher.
/// Called after fetching `/search-chunk-{id}.bin`.
#[wasm_bindgen]
pub fn oxidoc_search_load_chunk(data: &[u8]) -> Result<(), JsValue> {
    ENGINE.with(|e| {
        let mut borrow = e.borrow_mut();
        let engine = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;
        engine
            .load_chunk(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Get the chunk IDs needed for a given query.
/// Returns a JSON array of chunk IDs (e.g. "[0, 3, 7]").
#[wasm_bindgen]
pub fn oxidoc_search_needed_chunks(query: &str) -> Result<String, JsValue> {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;
        let ids = engine.needed_chunk_ids(query);
        serde_json::to_string(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Load the GGUF embedding model + pre-computed vectors to enable hybrid search.
///
/// Call with the GGUF model bytes and the vectors JSON string.
/// After this, `oxidoc_search_query_ai` will use RRF fusion of lexical + semantic.
#[cfg(feature = "semantic")]
#[wasm_bindgen]
pub fn oxidoc_search_load_semantic(model_data: &[u8], vectors_json: &str) -> Result<(), JsValue> {
    let vector_index = deserialize_vector_index(vectors_json.as_bytes())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let device = CpuDevice::new();

    let mut gguf = Gguf::from_bytes(model_data.to_vec())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse GGUF model: {}", e)))?;

    let pipeline = EmbeddingPipeline::<CpuRuntime, _>::from_gguf(&mut gguf, device.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to load embedding pipeline: {}", e)))?;

    let semantic_searcher = SemanticSearcher::new(pipeline, vector_index, device);

    ENGINE.with(|e| {
        let mut borrow = e.borrow_mut();
        let engine = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;
        engine.set_semantic(semantic_searcher);
        Ok(())
    })
}

/// Run a lexical-only search query and return JSON results.
#[wasm_bindgen]
pub fn oxidoc_search_query(query_text: &str, max_results: usize) -> Result<String, JsValue> {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;

        let query = SearchQuery {
            text: query_text.to_string(),
            max_results,
        };

        let results = lexical_query(engine, &query);
        serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Run a hybrid (lexical + semantic) search query via "Ask AI".
/// Returns JSON results using RRF fusion. Requires semantic model to be loaded.
#[cfg(feature = "semantic")]
#[wasm_bindgen]
pub fn oxidoc_search_query_ai(query_text: &str, max_results: usize) -> Result<String, JsValue> {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;

        let query = SearchQuery {
            text: query_text.to_string(),
            max_results,
        };

        CLIENT.with(|c| {
            let cborrow = c.borrow();
            let client = cborrow
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Client not initialized"))?;

            let results = engine
                .search(client, &query)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
        })
    })
}

/// Check if the search engine is initialized.
#[wasm_bindgen]
pub fn oxidoc_search_ready() -> bool {
    ENGINE.with(|e| e.borrow().is_some())
}

/// Check if semantic search is available.
#[cfg(feature = "semantic")]
#[wasm_bindgen]
pub fn oxidoc_search_has_semantic() -> bool {
    ENGINE.with(|e| e.borrow().as_ref().is_some_and(|eng| eng.has_semantic()))
}

/// Check if semantic search is available. Always false without the `semantic` feature.
#[cfg(not(feature = "semantic"))]
#[wasm_bindgen]
pub fn oxidoc_search_has_semantic() -> bool {
    false
}
