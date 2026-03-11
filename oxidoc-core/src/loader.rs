/// Generate the `oxidoc-loader.js` entry point script.
///
/// This is a <2KB JavaScript file that:
/// 1. Loads the core `oxidoc-registry.wasm` on every page
/// 2. Detects `api-playground` islands and lazy-loads `oxidoc-openapi.wasm`
/// 3. Detects search interactions and lazy-loads `oxidoc-search.wasm`
pub fn generate_loader_js() -> &'static str {
    r#"/* oxidoc-loader.js — Wasm island hydration loader */
(function(){
"use strict";

var REGISTRY_WASM = "/oxidoc-registry.wasm";
var OPENAPI_WASM = "/oxidoc-openapi.wasm";
var SEARCH_WASM = "/oxidoc-search.wasm";

var loaded = {};

function loadWasm(url) {
    if (loaded[url]) return loaded[url];
    loaded[url] = fetch(url)
        .then(function(r) {
            if (!r.ok) throw new Error("Failed to load " + url + ": " + r.status);
            return WebAssembly.instantiateStreaming(r);
        })
        .then(function(result) {
            if (result.instance.exports.__wasm_start) {
                result.instance.exports.__wasm_start();
            }
            return result.instance;
        })
        .catch(function(err) {
            console.warn("[oxidoc] Could not load " + url + ":", err.message);
            delete loaded[url];
        });
    return loaded[url];
}

function hasIsland(type) {
    return document.querySelector('oxidoc-island[data-island-type="' + type + '"]') !== null;
}

function init() {
    /* Core registry — always load */
    loadWasm(REGISTRY_WASM);

    /* Conditional: OpenAPI playground */
    if (hasIsland("api-playground")) {
        loadWasm(OPENAPI_WASM);
    }

    /* Conditional: Search — load on first interaction */
    var searchTrigger = document.querySelector("[data-oxidoc-search]");
    if (searchTrigger) {
        searchTrigger.addEventListener("click", function handler() {
            loadWasm(SEARCH_WASM);
            searchTrigger.removeEventListener("click", handler);
        }, { once: true });
    }
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
} else {
    init();
}
})();
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_contains_registry() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc-registry.wasm"));
    }

    #[test]
    fn loader_contains_conditional_loading() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc-openapi.wasm"));
        assert!(js.contains("oxidoc-search.wasm"));
        assert!(js.contains("api-playground"));
    }

    #[test]
    fn loader_is_small() {
        let js = generate_loader_js();
        assert!(
            js.len() < 2048,
            "Loader should be <2KB, got {} bytes",
            js.len()
        );
    }
}
