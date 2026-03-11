use wasm_bindgen::prelude::*;

/// Entry point for the Oxidoc Wasm registry.
/// Called when the Wasm module is loaded in the browser.
#[wasm_bindgen(start)]
pub fn start() {
    hydrate_islands();
    observe_mutations();
}

/// Scans the DOM for all `<oxidoc-island>` elements and hydrates them
/// with the appropriate component from `oxidoc-components`.
fn hydrate_islands() {
    let Some(window) = web_sys::window() else {
        web_sys::console::error_1(&"Oxidoc: unable to access window object".into());
        return;
    };
    let Some(document) = window.document() else {
        web_sys::console::error_1(&"Oxidoc: unable to access document".into());
        return;
    };

    let Ok(islands) = document.query_selector_all("oxidoc-island") else {
        web_sys::console::error_1(&"Oxidoc: querySelectorAll failed".into());
        return;
    };

    for i in 0..islands.length() {
        let Some(node) = islands.item(i) else {
            continue;
        };
        let Ok(element) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };

        hydrate_element(element);
    }
}

fn hydrate_element(element: web_sys::Element) {
    // Skip already-hydrated islands
    if element.get_attribute("data-hydrated").is_some() {
        return;
    }

    let Some(island_type) = element.get_attribute("data-island-type") else {
        return;
    };
    let props_json = element
        .get_attribute("data-props")
        .unwrap_or_else(|| "{}".into());

    match mount_island(&island_type, element.clone(), &props_json) {
        Ok(()) => {
            let _ = element.set_attribute("data-hydrated", "true");
        }
        Err(e) => {
            web_sys::console::error_1(
                &format!("Failed to mount island '{island_type}': {e}").into(),
            );
        }
    }
}

fn mount_island(
    island_type: &str,
    target: web_sys::Element,
    props_json: &str,
) -> Result<(), oxidoc_island::IslandError> {
    use oxidoc_island::OxidocIsland;

    match island_type {
        "accordion" => oxidoc_components::accordion::Accordion::mount(target, props_json),
        "callout" => oxidoc_components::callout::Callout::mount(target, props_json),
        "cardgrid" => oxidoc_components::card_grid::CardGrid::mount(target, props_json),
        "codeblock" => oxidoc_components::code_block::CodeBlock::mount(target, props_json),
        "tabs" => oxidoc_components::tabs::Tabs::mount(target, props_json),
        _ => {
            web_sys::console::warn_1(&format!("Unknown island type: {island_type}").into());
            Ok(())
        }
    }
}

/// Set up a MutationObserver to detect dynamically added `<oxidoc-island>` elements.
fn observe_mutations() {
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
                        hydrate_element(el.clone());
                    }
                    // Also check children of the added node
                    let nested = el
                        .query_selector_all("oxidoc-island")
                        .unwrap_or_else(|_| web_sys::NodeList::from(JsValue::NULL));
                    for k in 0..nested.length() {
                        if let Some(child) = nested.item(k)
                            && let Ok(child_el) = child.dyn_into::<web_sys::Element>()
                        {
                            hydrate_element(child_el);
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
