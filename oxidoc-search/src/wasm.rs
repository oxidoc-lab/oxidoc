//! Wasm-bindgen exports for the search dialog.
//!
//! The JS search dialog calls these functions instead of implementing its own search.
//! Flow: JS fetches index JSON → calls `oxidoc_search_init` → calls `oxidoc_search_query` per keystroke.

use crate::lexical::LexicalSearcher;
use crate::types::SearchQuery;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static SEARCHER: RefCell<Option<LexicalSearcher>> = const { RefCell::new(None) };
}

/// Initialize the search engine with the lexical index JSON.
/// Called once after fetching `/search-lexical.json`.
#[wasm_bindgen]
pub fn oxidoc_search_init(json: &str) -> Result<(), JsValue> {
    let data = json.as_bytes();
    let searcher =
        LexicalSearcher::from_bytes(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
    SEARCHER.with(|s| {
        *s.borrow_mut() = Some(searcher);
    });
    Ok(())
}

/// Run a search query and return JSON results.
/// Returns a JSON array of `{ title, path, snippet, score, kind, page_title }`.
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
