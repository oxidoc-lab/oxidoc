pub mod accordion;
pub mod callout;
pub mod card_grid;
pub mod code_block;
pub mod tabs;

use oxidoc_island::IslandError;
use serde::de::DeserializeOwned;
use wasm_bindgen::JsCast;

/// Shared mount helper: deserialize props, cast target, and mount a Leptos view.
pub fn mount_component<P: DeserializeOwned + 'static, V: leptos::IntoView + 'static>(
    target: web_sys::Element,
    props_json: &str,
    view_fn: impl FnOnce(P) -> V + 'static,
) -> Result<(), IslandError> {
    let props: P = serde_json::from_str(props_json)?;
    let html_el: web_sys::HtmlElement = target.unchecked_into();
    let _ = leptos::mount::mount_to(html_el, move || view_fn(props));
    Ok(())
}
