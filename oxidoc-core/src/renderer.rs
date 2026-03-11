use serde_json;
use std::collections::HashMap;

/// Converts an RDX component tag and its attributes into an `<oxidoc-island>` placeholder.
pub fn render_island(tag: &str, attributes: &HashMap<String, String>) -> String {
    let props_json = serde_json::to_string(attributes).unwrap_or_else(|_| "{}".into());
    format!(
        r#"<oxidoc-island data-island-type="{}" data-props='{}'></oxidoc-island>"#,
        tag.to_lowercase(),
        props_json,
    )
}

/// Renders a Vanilla Web Component passthrough for tags registered in `[components.custom]`.
/// Bypasses the Wasm island pipeline entirely.
pub fn render_web_component(
    tag: &str,
    attributes: &HashMap<String, String>,
    js_src: &str,
) -> String {
    let attrs: String = attributes
        .iter()
        .map(|(k, v)| format!(r#" {k}="{v}""#))
        .collect();
    format!(r#"<{tag}{attrs}></{tag}><script src="{js_src}" type="module" async></script>"#)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_island_produces_valid_placeholder() {
        let mut attrs = HashMap::new();
        attrs.insert("type".into(), "warning".into());
        let html = render_island("Callout", &attrs);
        assert!(html.contains(r#"data-island-type="callout""#));
        assert!(html.contains(r#"data-props="#));
    }

    #[test]
    fn render_web_component_produces_custom_element() {
        let mut attrs = HashMap::new();
        attrs.insert("variant".into(), "dark".into());
        let html = render_web_component("PromoBanner", &attrs, "assets/js/promo-banner.js");
        assert!(html.contains("<PromoBanner"));
        assert!(html.contains("</PromoBanner>"));
        assert!(html.contains(r#"src="assets/js/promo-banner.js""#));
        assert!(html.contains(r#"type="module""#));
    }
}
