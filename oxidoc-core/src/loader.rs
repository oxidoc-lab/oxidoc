/// Generate the `oxidoc-loader.js` entry point script.
///
/// This is a <2KB JavaScript file that:
/// 1. Loads the core `oxidoc-registry.wasm` on every page
/// 2. Detects `api-playground` islands and lazy-loads `oxidoc-openapi.wasm`
/// 3. Detects search interactions and lazy-loads `oxidoc-search.wasm`
pub fn generate_loader_js() -> &'static str {
    r#"(function(){var L="/oxidoc-registry.wasm",O="/oxidoc-openapi.wasm",S="/oxidoc-search.wasm",l={},d=0;function w(u){if(l[u])return l[u];l[u]=fetch(u,{cache:"default"}).then(r=>r.ok?WebAssembly.instantiateStreaming(r):Promise.reject(""+r.status)).then(i=>{i.instance.exports.__wasm_start&&i.instance.exports.__wasm_start();return i}).catch(e=>{d=1;let n=document.querySelector("[data-oxidoc-banner]");if(!n){n=document.createElement("div");n.setAttribute("data-oxidoc-banner","");n.style.cssText="position:fixed;top:0;left:0;right:0;background:#fee2e2;border:1px solid #fca5a5;padding:.75rem 1rem;font-size:.875rem;color:#991b1b;z-index:999;text:center";n.textContent="Interactive features unavailable.";document.body.insertBefore(n,document.body.firstChild)}delete l[u]});return l[u]}var c=document.querySelectorAll("oxidoc-island");for(let i=0;i<c.length;i++)c[i].classList.add("oxidoc-loading");w(L);document.querySelector('oxidoc-island[data-island-type="api-playground"]')&&w(O);let t=document.querySelector("[data-oxidoc-search]");t&&t.addEventListener("click",()=>{w(S)},{once:1})})();"#
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

    #[test]
    fn loader_has_error_recovery() {
        let js = generate_loader_js();
        assert!(js.contains("data-oxidoc-banner"));
        assert!(js.contains("Interactive features"));
    }

    #[test]
    fn loader_has_loading_states() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc-island"));
        assert!(js.contains("oxidoc-loading"));
        assert!(js.contains("classList.add"));
    }

    #[test]
    fn loader_has_cache_hint() {
        let js = generate_loader_js();
        assert!(js.contains(r#"cache:"default""#));
    }
}
