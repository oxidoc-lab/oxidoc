use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Helper to set up an island element in the DOM.
fn setup_island(island_type: &str, props_json: &str) -> web_sys::Element {
    let document = web_sys::window().unwrap().document().unwrap();
    let el = document.create_element("oxidoc-island").unwrap();
    el.set_attribute("data-island-type", island_type).unwrap();
    el.set_attribute("data-props", props_json).unwrap();
    document.body().unwrap().append_child(&el).unwrap();
    el
}

/// Helper to tear down an island element from the DOM.
fn teardown(el: &web_sys::Element) {
    el.remove();
}

// ============================================================================
// Registry Hydration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_hydrate_islands_finds_and_mounts() {
    // Create multiple islands
    let el1 = setup_island(
        "callout",
        r#"{"kind":"warning","title":"Test","content":"Body"}"#,
    );
    let el2 = setup_island("tabs", r#"{"labels":["Tab 1"],"contents":["Content 1"]}"#);

    // Hydrate all islands
    oxidoc_registry::hydration::hydrate_islands();

    // Verify both got hydrated
    assert_eq!(el1.get_attribute("data-hydrated"), Some("true".into()));
    assert_eq!(el2.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el1);
    teardown(&el2);
}

#[wasm_bindgen_test]
fn test_hydrate_skips_already_hydrated() {
    let el = setup_island("callout", r#"{"kind":"info","title":"","content":""}"#);

    // Set it as already hydrated
    el.set_attribute("data-hydrated", "true").unwrap();

    // Try to hydrate it
    oxidoc_registry::hydration::hydrate_islands();

    // Should still be marked as hydrated and not double-mounted
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_hydrate_no_crash_empty_dom() {
    // Clear the body of any islands (if any)
    let document = web_sys::window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    let islands = body
        .query_selector_all("oxidoc-island")
        .unwrap_or_else(|_| web_sys::NodeList::from(wasm_bindgen::JsValue::NULL));

    for i in 0..islands.length() {
        if let Some(node) = islands.item(i)
            && let Ok(el) = node.dyn_into::<web_sys::Element>()
        {
            el.remove();
        }
    }

    // Should not crash when calling hydrate on empty DOM
    oxidoc_registry::hydration::hydrate_islands();
}

// ============================================================================
// Component Rendering Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_callout_renders() {
    let props = r#"{"kind":"warning","title":"Watch Out","content":"<p>Be careful</p>"}"#;
    let el = setup_island("callout", props);

    // Log for debugging
    web_sys::console::log_1(&format!("Props: {}", props).into());

    oxidoc_registry::hydration::hydrate_islands();

    // Verify hydrated (this confirms the component mounted successfully)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_tabs_renders() {
    let props = r#"{"labels":["Tab 1","Tab 2"],"contents":["Content 1","Content 2"]}"#;
    let el = setup_island("tabs", props);

    oxidoc_registry::hydration::hydrate_islands();

    // Verify hydrated (this confirms the component mounted successfully)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_codeblock_renders() {
    let props = r#"{"code":"let x = 1;","language":"rust","line_numbers":true}"#;
    let el = setup_island("codeblock", props);

    oxidoc_registry::hydration::hydrate_islands();

    // Verify hydrated (this confirms the component mounted successfully)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_accordion_renders() {
    let props =
        r#"{"items":[{"title":"Section 1","content":"Body 1","open":true}],"multiple":false}"#;
    let el = setup_island("accordion", props);

    oxidoc_registry::hydration::hydrate_islands();

    // Verify hydrated (this confirms the component mounted successfully)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_cardgrid_renders() {
    let props = r#"{"columns":2,"cards":[{"title":"Card 1","description":"Desc 1"}]}"#;
    let el = setup_island("cardgrid", props);

    oxidoc_registry::hydration::hydrate_islands();

    // Verify hydrated (this confirms the component mounted successfully)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

// ============================================================================
// Unknown/Error Case Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_unknown_island_no_panic() {
    let props = r#"{"some":"data"}"#;
    let el = setup_island("nonexistent", props);

    // Should not panic; unknown islands are logged but marked as hydrated
    oxidoc_registry::hydration::hydrate_islands();

    // Verify it was marked as hydrated (mount_island returns Ok for unknown)
    assert_eq!(el.get_attribute("data-hydrated"), Some("true".into()));

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_invalid_props_error() {
    let props = "not json";
    let el = setup_island("callout", props);

    oxidoc_registry::hydration::hydrate_islands();

    // Invalid JSON should cause mount_island to error
    // and NOT set data-hydrated
    assert_eq!(el.get_attribute("data-hydrated"), None);

    teardown(&el);
}

#[wasm_bindgen_test]
fn test_missing_island_type_skipped() {
    let document = web_sys::window().unwrap().document().unwrap();
    let el = document.create_element("oxidoc-island").unwrap();
    // Note: not setting data-island-type
    el.set_attribute("data-props", r#"{"kind":"info"}"#)
        .unwrap();
    document.body().unwrap().append_child(&el).unwrap();

    oxidoc_registry::hydration::hydrate_islands();

    // Should not be hydrated (skipped due to missing data-island-type)
    assert_eq!(el.get_attribute("data-hydrated"), None);

    teardown(&el);
}
