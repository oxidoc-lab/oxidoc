use wasm_bindgen::JsValue;

/// Error type for island mount failures.
#[derive(Debug)]
pub struct IslandError {
    pub message: String,
}

impl std::fmt::Display for IslandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Island error: {}", self.message)
    }
}

impl From<serde_json::Error> for IslandError {
    fn from(e: serde_json::Error) -> Self {
        Self {
            message: format!("Props deserialization failed: {e}"),
        }
    }
}

impl From<JsValue> for IslandError {
    fn from(e: JsValue) -> Self {
        Self {
            message: format!("DOM error: {e:?}"),
        }
    }
}

/// The core trait that all custom and built-in island components must implement.
///
/// Implement this trait to create a custom Oxidoc island plugin.
/// The registry will call `mount` when it finds a matching `<oxidoc-island>` tag in the DOM.
pub trait OxidocIsland {
    /// The identifier used in `data-island-type` attribute (e.g., `"callout"`, `"tabs"`).
    fn island_type() -> &'static str;

    /// Mount the component into the given DOM element, hydrating from the serialized JSON props.
    ///
    /// Returns `Err` if props deserialization fails or DOM manipulation errors occur.
    fn mount(target: web_sys::Element, props_json: &str) -> Result<(), IslandError>;

    /// Unmount the component from the DOM element, cleaning up event listeners and state.
    ///
    /// Default implementation clears the element's innerHTML.
    fn unmount(target: web_sys::Element) -> Result<(), IslandError> {
        target.set_inner_html("");
        Ok(())
    }
}
