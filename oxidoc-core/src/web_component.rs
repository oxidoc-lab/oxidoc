use rdx_ast::AttributeValue;
use std::fmt::Write;

/// Render a Vanilla Web Component passthrough.
pub(crate) fn render_web_component(
    tag: &str,
    attributes: &[rdx_ast::AttributeNode],
    js_src: &str,
    out: &mut String,
) {
    let _ = write!(out, "<{tag}");
    for attr in attributes {
        let val = crate::utils::html_escape(&attribute_value_to_string(&attr.value));
        let _ = write!(out, r#" {}="{val}""#, attr.name);
    }
    let _ = write!(
        out,
        r#"></{tag}><script src="{}" type="module" async></script>"#,
        crate::utils::html_escape(js_src)
    );
}

fn attribute_value_to_string(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Null => String::new(),
        AttributeValue::Bool(b) => b.to_string(),
        AttributeValue::Number(n) => n.to_string(),
        AttributeValue::String(s) => s.clone(),
        AttributeValue::Array(a) => serde_json::to_string(a).unwrap_or_default(),
        AttributeValue::Object(o) => serde_json::to_string(o).unwrap_or_default(),
        AttributeValue::Variable(v) => format!("${{{}}}", v.path),
    }
}
