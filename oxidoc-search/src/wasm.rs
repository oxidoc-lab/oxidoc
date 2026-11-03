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
//!
//! Sites with ranked records also fetch `/search-records.bin` and call
//! `oxidoc_search_load_records(data)`. Then `oxidoc_search_query_filtered`,
//! `oxidoc_search_query_ai_filtered` and `oxidoc_search_facets` take filters as
//! JSON, such as `{"kind":["library"],"topic":["nlp","vision"]}`.

#[cfg(feature = "semantic")]
use crate::engine::SearchEngine;
#[cfg(feature = "semantic")]
use crate::index::deserialize_vector_index;
use crate::lexical::LexicalSearcher;
use crate::records::{Filter, parse_filters_json};
#[cfg(feature = "semantic")]
use crate::semantic::SemanticSearcher;
use crate::types::{SearchQuery, SearchResult as DocResult};
#[cfg(feature = "semantic")]
use boostr::format::Gguf;
#[cfg(feature = "semantic")]
use boostr::{CpuClient, CpuDevice, CpuRuntime, EmbeddingPipeline};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// The hybrid search engine: lexical always, semantic when model is loaded.
#[cfg(feature = "semantic")]
type Engine = SearchEngine<CpuRuntime>;

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

/// Load `search-records.bin` to enable field weights, boost, filters and facets.
/// Called after `oxidoc_search_init`.
#[wasm_bindgen]
pub fn oxidoc_search_load_records(data: &[u8]) -> Result<(), JsValue> {
    ENGINE.with(|e| {
        let mut borrow = e.borrow_mut();
        let engine = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;
        engine
            .load_records(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Parses the filters JSON argument of an export.
fn filters_arg(filters_json: &str) -> Result<Vec<Filter>, JsValue> {
    parse_filters_json(filters_json).map_err(|e| JsValue::from_str(&e.to_string()))
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

    let pipeline = EmbeddingPipeline::<CpuRuntime>::from_gguf(&mut gguf, device.clone())
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
            filters: Vec::new(),
        };

        let results = lexical_query(engine, &query);
        serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Run a lexical search limited by tag filters and return JSON results.
///
/// `filters_json` is an object of string arrays: OR within a key, AND across keys.
/// Malformed JSON returns an error.
#[wasm_bindgen]
pub fn oxidoc_search_query_filtered(
    query_text: &str,
    max_results: usize,
    filters_json: &str,
) -> Result<String, JsValue> {
    let filters = filters_arg(filters_json)?;
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;

        let query = SearchQuery {
            text: query_text.to_string(),
            max_results,
            filters,
        };

        let results = lexical_query(engine, &query);
        serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Count tag values over the documents that match a query, as JSON.
///
/// Returns `[[key, [[value, count], ...]], ...]`. Keys are sorted. Values come by
/// count, highest first. Malformed filters JSON returns an error.
#[wasm_bindgen]
pub fn oxidoc_search_facets(query_text: &str, filters_json: &str) -> Result<String, JsValue> {
    let filters = filters_arg(filters_json)?;
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;
        let facets = engine.facets(query_text, &filters);
        serde_json::to_string(&facets).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Run a hybrid (lexical + semantic) search query via "Ask AI".
/// Returns JSON results using RRF fusion. Requires semantic model to be loaded.
#[cfg(feature = "semantic")]
#[wasm_bindgen]
pub fn oxidoc_search_query_ai(query_text: &str, max_results: usize) -> Result<String, JsValue> {
    hybrid_query(SearchQuery {
        text: query_text.to_string(),
        max_results,
        filters: Vec::new(),
    })
}

/// Run a hybrid search limited by tag filters and return JSON results.
///
/// Filters apply to both rankers before fusion. Malformed filters JSON returns an error.
#[cfg(feature = "semantic")]
#[wasm_bindgen]
pub fn oxidoc_search_query_ai_filtered(
    query_text: &str,
    max_results: usize,
    filters_json: &str,
) -> Result<String, JsValue> {
    hybrid_query(SearchQuery {
        text: query_text.to_string(),
        max_results,
        filters: filters_arg(filters_json)?,
    })
}

/// Runs `query` on the hybrid engine and returns JSON results.
#[cfg(feature = "semantic")]
fn hybrid_query(query: SearchQuery) -> Result<String, JsValue> {
    ENGINE.with(|e| {
        let borrow = e.borrow();
        let engine = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search engine not initialized"))?;

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
