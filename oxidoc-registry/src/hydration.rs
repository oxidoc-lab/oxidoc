use wasm_bindgen::JsCast;

/// Scans the DOM for all `<oxidoc-island>` elements and hydrates them
/// with the appropriate component from `oxidoc-components`.
pub fn hydrate_islands() {
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

pub fn hydrate_element(element: web_sys::Element) {
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
        "api-playground" => oxidoc_openapi::ApiPlayground::mount(target, props_json),
        _ => {
            web_sys::console::warn_1(&format!("Unknown island type: {island_type}").into());
            Ok(())
        }
    }
}
