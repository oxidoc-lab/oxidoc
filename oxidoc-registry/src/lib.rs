pub mod hydration;
pub mod observer;

use wasm_bindgen::prelude::*;

/// Entry point for the Oxidoc Wasm registry.
/// Called when the Wasm module is loaded in the browser.
#[wasm_bindgen(start)]
pub fn start() {
    hydration::hydrate_islands();
    observer::observe_mutations();
}
