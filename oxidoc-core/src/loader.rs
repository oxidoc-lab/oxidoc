/// Generate the `oxidoc-loader.js` entry point script.
///
/// This is a <2KB JavaScript file that:
/// 1. Loads the core `oxidoc_registry.js` on every page
/// 2. Detects `api-playground` islands and lazy-loads `oxidoc_openapi.js`
/// 3. Detects search interactions and lazy-loads `oxidoc_search.js`
///
/// A cache-busting version hash is appended to all import URLs to prevent
/// stale cached Wasm/JS files from breaking hydration after updates.
pub fn generate_loader_js(cache_bust: &str) -> String {
    let v = if cache_bust.is_empty() {
        String::new()
    } else {
        format!("?v={cache_bust}")
    };
    format!(
        r#"(function(){{var L="/oxidoc_registry.js{v}",O="/oxidoc_openapi.js{v}",S="/oxidoc_search.js{v}",l={{}};function w(u){{if(l[u])return l[u];l[u]=import(u).then(m=>m.default().then(()=>m)).catch(e=>{{console.error("[oxidoc] Failed to load "+u+":",e);let n=document.querySelector("[data-oxidoc-banner]");if(!n){{n=document.createElement("div");n.setAttribute("data-oxidoc-banner","");n.style.cssText="position:fixed;top:0;left:0;right:0;background:#fee2e2;border:1px solid #fca5a5;padding:.75rem 1rem;font-size:.875rem;color:#991b1b;z-index:9999;text-align:center";n.innerHTML=""}}n.innerHTML+='<div>[oxidoc] Failed to load <code>'+u+'</code>: '+(e.message||e)+'</div>';if(!n.parentNode)document.body.insertBefore(n,document.body.firstChild);delete l[u]}});return l[u]}}var c=document.querySelectorAll("oxidoc-island");if(c.length>0){{for(let i=0;i<c.length;i++)c[i].classList.add("oxidoc-loading");w(L);document.querySelector('oxidoc-island[data-island-type="api-playground"]')&&w(O)}}window.__oxidoc_search=function(){{return w(S)}}}})();"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_contains_registry() {
        let js = generate_loader_js("test");
        assert!(js.contains("oxidoc_registry.js"));
    }

    #[test]
    fn loader_contains_conditional_loading() {
        let js = generate_loader_js("test");
        assert!(js.contains("oxidoc_openapi.js"));
        assert!(js.contains("oxidoc_search.js"));
        assert!(js.contains("api-playground"));
    }

    #[test]
    fn loader_is_small() {
        let js = generate_loader_js("test");
        assert!(
            js.len() < 2048,
            "Loader should be <2KB, got {} bytes",
            js.len()
        );
    }

    #[test]
    fn loader_has_error_recovery() {
        let js = generate_loader_js("test");
        assert!(js.contains("data-oxidoc-banner"));
        assert!(js.contains("Failed to load"));
    }

    #[test]
    fn loader_has_loading_states() {
        let js = generate_loader_js("test");
        assert!(js.contains("oxidoc-island"));
        assert!(js.contains("oxidoc-loading"));
        assert!(js.contains("classList.add"));
    }

    #[test]
    fn loader_shows_specific_error() {
        let js = generate_loader_js("test");
        assert!(js.contains("console.error"));
        assert!(js.contains("e.message"));
    }
}
