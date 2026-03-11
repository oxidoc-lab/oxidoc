//! ApiPlayground island component with Shadow DOM isolation.

use super::types::ApiPlaygroundProps;
use super::view;
use crate::styles;
use oxidoc_island::{IslandError, OxidocIsland};
use wasm_bindgen::JsCast;

pub struct ApiPlayground;

impl OxidocIsland for ApiPlayground {
    fn island_type() -> &'static str {
        "api-playground"
    }

    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError> {
        let props: ApiPlaygroundProps = serde_json::from_str(props_json)?;

        // Attach Shadow DOM for CSS isolation
        let shadow = oxidoc_island::shadow::attach_shadow(&target)?;
        oxidoc_island::shadow::inject_shadow_styles(&shadow, &styles::get_styles())?;

        // Create a container div in the shadow root
        let window = web_sys::window().ok_or_else(|| IslandError {
            message: "Window object not available".into(),
        })?;

        let document = window.document().ok_or_else(|| IslandError {
            message: "Document object not available".into(),
        })?;

        let container = document.create_element("div").map_err(|e| IslandError {
            message: format!("Failed to create container: {e:?}"),
        })?;

        container
            .set_attribute("class", "api-playground")
            .map_err(|e| IslandError {
                message: format!("Failed to set class: {e:?}"),
            })?;

        shadow.append_child(&container).map_err(|e| IslandError {
            message: format!("Failed to append container to shadow: {e:?}"),
        })?;

        // Mount Leptos view into the container
        let html_el: web_sys::HtmlElement = container.unchecked_into();
        let _ = leptos::mount::mount_to(html_el, move || view::playground_view(props.clone()));

        Ok(())
    }
}
