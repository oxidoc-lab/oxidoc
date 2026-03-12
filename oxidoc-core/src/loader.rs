/// Generate the `oxidoc-loader.js` entry point script.
///
/// This is a <2KB JavaScript file that:
/// 1. Loads the core `oxidoc_registry.js` on every page
/// 2. Detects `api-playground` islands and lazy-loads `oxidoc_openapi.js`
/// 3. Detects search interactions and lazy-loads `oxidoc_search.js`
pub fn generate_loader_js() -> &'static str {
    r#"(function(){var L="/oxidoc_registry.js",O="/oxidoc_openapi.js",S="/oxidoc_search.js",l={};function w(u){if(l[u])return l[u];l[u]=import(u).then(m=>m.default()).then(m=>{if(m&&m.start)m.start();return m}).catch(e=>{console.error("[oxidoc] Failed to load "+u+":",e);let n=document.querySelector("[data-oxidoc-banner]");if(!n){n=document.createElement("div");n.setAttribute("data-oxidoc-banner","");n.style.cssText="position:fixed;top:0;left:0;right:0;background:#fee2e2;border:1px solid #fca5a5;padding:.75rem 1rem;font-size:.875rem;color:#991b1b;z-index:9999;text-align:center";n.innerHTML=""}n.innerHTML+='<div>[oxidoc] Failed to load <code>'+u+'</code>: '+(e.message||e)+'</div>';if(!n.parentNode)document.body.insertBefore(n,document.body.firstChild);delete l[u]});return l[u]}var c=document.querySelectorAll("oxidoc-island");if(c.length>0){for(let i=0;i<c.length;i++)c[i].classList.add("oxidoc-loading");w(L);document.querySelector('oxidoc-island[data-island-type="api-playground"]')&&w(O)}let t=document.querySelector("[data-oxidoc-search]");t&&t.addEventListener("click",()=>{w(S)},{once:1})})();"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_contains_registry() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc_registry.js"));
    }

    #[test]
    fn loader_contains_conditional_loading() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc_openapi.js"));
        assert!(js.contains("oxidoc_search.js"));
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
        assert!(js.contains("Failed to load"));
    }

    #[test]
    fn loader_has_loading_states() {
        let js = generate_loader_js();
        assert!(js.contains("oxidoc-island"));
        assert!(js.contains("oxidoc-loading"));
        assert!(js.contains("classList.add"));
    }

    #[test]
    fn loader_shows_specific_error() {
        let js = generate_loader_js();
        assert!(js.contains("console.error"));
        assert!(js.contains("e.message"));
    }
}
