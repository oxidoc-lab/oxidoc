use wasm_bindgen::prelude::*;

/// Set up a MutationObserver to detect dynamically added `<oxidoc-island>` elements.
pub fn observe_mutations() {
    let callback = Closure::<dyn Fn(js_sys::Array)>::new(move |mutations: js_sys::Array| {
        for i in 0..mutations.length() {
            let mutation: web_sys::MutationRecord = mutations.get(i).unchecked_into();
            let added = mutation.added_nodes();
            for j in 0..added.length() {
                let Some(node) = added.item(j) else {
                    continue;
                };
                // Check if the added node itself is an island
                if let Ok(el) = node.clone().dyn_into::<web_sys::Element>() {
                    if el.tag_name().eq_ignore_ascii_case("OXIDOC-ISLAND") {
                        crate::hydration::hydrate_element(el.clone());
                    }
                    // Also check children of the added node
                    let nested = el
                        .query_selector_all("oxidoc-island")
                        .unwrap_or_else(|_| web_sys::NodeList::from(JsValue::NULL));
                    for k in 0..nested.length() {
                        if let Some(child) = nested.item(k)
                            && let Ok(child_el) = child.dyn_into::<web_sys::Element>()
                        {
                            crate::hydration::hydrate_element(child_el);
                        }
                    }
                }
            }
        }
    });

    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(&"Oxidoc: unable to access window for mutations".into());
        return;
    };
    let Some(document) = window.document() else {
        web_sys::console::error_1(&"Oxidoc: unable to access document for mutations".into());
        return;
    };
    let Some(root) = document.document_element() else {
        web_sys::console::error_1(&"Oxidoc: unable to access document root".into());
        return;
    };

    let Ok(observer) = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref()) else {
        web_sys::console::error_1(&"Oxidoc: MutationObserver creation failed".into());
        return;
    };

    let config = web_sys::MutationObserverInit::new();
    config.set_child_list(true);
    config.set_subtree(true);

    let _ = observer.observe_with_options(&root, &config);

    // Leak the closure so it stays alive for the page lifetime
    callback.forget();
}
