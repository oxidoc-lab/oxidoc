/// The core trait that all custom and built-in island components must implement.
///
/// Implement this trait to create a custom Oxidoc island plugin.
/// The registry will call `mount` when it finds a matching `<oxidoc-island>` tag in the DOM.
pub trait OxidocIsland {
    /// The identifier used in `data-island-type` attribute (e.g., `"callout"`, `"tabs"`).
    fn island_type() -> &'static str;

    /// Mount the component into the given DOM element, hydrating from the serialized JSON props.
    fn mount(target: web_sys::Element, props_json: &str);
}
