//! Interactive API Playground island component with Shadow DOM isolation.

mod view;

use crate::styles;
use oxidoc_island::{IslandError, OxidocIsland};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

/// Props for the API Playground component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiPlaygroundProps {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub parameters: Vec<ParameterDef>,
    #[serde(default)]
    pub request_body_schema: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

/// Definition of an API parameter from the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(rename = "type", default = "default_type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
}

fn default_type() -> String {
    "string".into()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_def_default_type() {
        let param = ParameterDef {
            name: "test".into(),
            location: "query".into(),
            param_type: default_type(),
            required: false,
        };
        assert_eq!(param.param_type, "string");
    }
}
