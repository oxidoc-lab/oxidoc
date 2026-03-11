use wasm_bindgen::prelude::*;

/// Entry point for the Oxidoc Wasm registry.
/// Called when the Wasm module is loaded in the browser.
#[wasm_bindgen(start)]
pub fn start() {
    hydrate_islands();
}

/// Scans the DOM for all `<oxidoc-island>` elements and hydrates them
/// with the appropriate component from `oxidoc-components`.
fn hydrate_islands() {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    let islands = document
        .query_selector_all("oxidoc-island")
        .expect("querySelectorAll failed");

    for i in 0..islands.length() {
        let Some(node) = islands.item(i) else {
            continue;
        };
        let Ok(element) = node.dyn_into::<web_sys::Element>() else {
            continue;
        };

        let Some(island_type) = element.get_attribute("data-island-type") else {
            continue;
        };
        let props_json = element
            .get_attribute("data-props")
            .unwrap_or_else(|| "{}".into());

        mount_island(&island_type, element, &props_json);
    }
}

fn mount_island(island_type: &str, target: web_sys::Element, props_json: &str) {
    use oxidoc_island::OxidocIsland;

    match island_type {
        "accordion" => oxidoc_components::accordion::Accordion::mount(target, props_json),
        "callout" => oxidoc_components::callout::Callout::mount(target, props_json),
        "cardgrid" => oxidoc_components::card_grid::CardGrid::mount(target, props_json),
        "codeblock" => oxidoc_components::code_block::CodeBlock::mount(target, props_json),
        "tabs" => oxidoc_components::tabs::Tabs::mount(target, props_json),
        _ => {
            web_sys::console::warn_1(&format!("Unknown island type: {island_type}").into());
        }
    }
}
