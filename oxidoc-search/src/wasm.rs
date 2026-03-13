//! Wasm-bindgen exports for the search dialog.
//!
//! The JS search dialog calls these functions instead of implementing its own search.
//! Flow: JS fetches search-meta.bin → calls `oxidoc_search_init` → fetches chunks as needed
//! → calls `oxidoc_search_load_chunk` → calls `oxidoc_search_query` per keystroke.

use crate::lexical::LexicalSearcher;
use crate::types::SearchQuery;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static SEARCHER: RefCell<Option<LexicalSearcher>> = const { RefCell::new(None) };
}

/// Initialize the search engine with the metadata binary (documents + chunk manifest).
/// Called once after fetching `/search-meta.bin`.
#[wasm_bindgen]
pub fn oxidoc_search_init(data: &[u8]) -> Result<(), JsValue> {
    let searcher =
        LexicalSearcher::from_metadata(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
    SEARCHER.with(|s| {
        *s.borrow_mut() = Some(searcher);
    });
    Ok(())
}

/// Load a chunk's postings into the searcher.
/// Called after fetching `/search-chunk-{id}.bin`.
#[wasm_bindgen]
pub fn oxidoc_search_load_chunk(data: &[u8]) -> Result<(), JsValue> {
    SEARCHER.with(|s| {
        let mut borrow = s.borrow_mut();
        let searcher = borrow
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Search index not initialized"))?;
        searcher
            .load_chunk(data)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Get the chunk IDs needed for a given query.
/// Returns a JSON array of chunk IDs (e.g. "[0, 3, 7]").
#[wasm_bindgen]
pub fn oxidoc_search_needed_chunks(query: &str) -> Result<String, JsValue> {
    SEARCHER.with(|s| {
        let borrow = s.borrow();
        let searcher = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search index not initialized"))?;
        let ids = searcher.needed_chunk_ids(query);
        serde_json::to_string(&ids).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Run a search query and return JSON results.
/// Returns a JSON array of `{ title, path, snippet, score, source, breadcrumb, anchor, highlight_terms }`.
#[wasm_bindgen]
pub fn oxidoc_search_query(query_text: &str, max_results: usize) -> Result<String, JsValue> {
    SEARCHER.with(|s| {
        let borrow = s.borrow();
        let searcher = borrow
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Search index not initialized"))?;

        let query = SearchQuery {
            text: query_text.to_string(),
            max_results,
        };

        let results = searcher.search(&query);
        serde_json::to_string(&results).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Check if the search engine is initialized.
#[wasm_bindgen]
pub fn oxidoc_search_ready() -> bool {
    SEARCHER.with(|s| s.borrow().is_some())
}
