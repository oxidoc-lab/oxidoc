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
//! 3. Optionally: fetch `/search-model.gguf` → `oxidoc_search_load_model(data)`
//!    and `/search-vectors.json` → `oxidoc_search_load_vectors(json)`
//! 4. `oxidoc_search_query(text, max)` → JSON results (hybrid if model loaded, lexical otherwise)

use crate::engine::SearchEngine;
use crate::index::deserialize_vector_index;
use crate::lexical::LexicalSearcher;
use crate::semantic::SemanticSearcher;
use crate::types::SearchQuery;
use boostr::format::Gguf;
use boostr::{CpuClient, CpuDevice, CpuRuntime, EmbeddingPipeline, GgufTokenizer};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

/// The hybrid search engine: lexical always, semantic when model is loaded.
type HybridEngine = SearchEngine<CpuRuntime, GgufTokenizer>;

thread_local! {
    static ENGINE: RefCell<Option<HybridEngine>> = const { RefCell::new(None) };
    static CLIENT: RefCell<Option<CpuClient>> = const { RefCell::new(None) };
}

/// Initialize the search engine with the metadata binary (documents + chunk manifest).
/// Called once after fetching `/search-meta.bin`.
#[wasm_bindgen]
pub fn oxidoc_search_init(data: &[u8]) -> Result<(), JsValue> {
    let searcher =
        LexicalSearcher::from_metadata(data).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let engine = SearchEngine::new(searcher, None);

    let device = CpuDevice::new();
    let client = CpuClient::new(device);

    ENGINE.with(|e| {
        *e.borrow_mut() = Some(engine);
    });
    CLIENT.with(|c| {
        *c.borrow_mut() = Some(client);
    });
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
/// After this, `oxidoc_search_query` will use RRF fusion of lexical + semantic.
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

        let results = engine.search_lexical(&query);
        serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Run a hybrid (lexical + semantic) search query via "Ask AI".
/// Returns JSON results using RRF fusion. Requires semantic model to be loaded.
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
#[wasm_bindgen]
pub fn oxidoc_search_has_semantic() -> bool {
    ENGINE.with(|e| {
        e.borrow()
            .as_ref()
            .map(|eng| eng.has_semantic())
            .unwrap_or(false)
    })
}
