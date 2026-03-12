use rdx_ast::Node;
use std::collections::HashMap;

use crate::renderer::{RenderCtx, render_children};
use crate::static_render::prop_str;

/// Render children to a standalone HTML string (for embedding in island props).
pub(crate) fn render_children_to_string(children: &[Node], ctx: &RenderCtx<'_>) -> String {
    let mut buf = String::new();
    render_children(children, &mut buf, ctx);
    buf
}

/// Collect plain text from AST children (no HTML tags).
pub(crate) fn collect_text(children: &[Node]) -> String {
    crate::utils::extract_plain_text_from_nodes(children)
}

/// Build the proper props JSON that the wasm component expects.
/// This bridges the gap between RDX AST attributes and component prop structs.
pub(crate) fn build_hydration_props(
    name: &str,
    attrs: &HashMap<String, serde_json::Value>,
    children: &[Node],
    raw_content: &str,
    ctx: &RenderCtx<'_>,
) -> serde_json::Value {
    match name {
        "Tabs" => build_tabs_props(attrs, children, ctx),
        "Accordion" => build_accordion_props(attrs, children, ctx),
        "CodeBlock" => build_code_block_props(attrs, children, raw_content),
        // Tab shouldn't appear at top level, but handle gracefully
        "Tab" => serde_json::to_value(attrs).unwrap_or_default(),
        _ => serde_json::to_value(attrs).unwrap_or_default(),
    }
}

fn build_tabs_props(
    attrs: &HashMap<String, serde_json::Value>,
    children: &[Node],
    ctx: &RenderCtx<'_>,
) -> serde_json::Value {
    let mut labels = Vec::new();
    let mut contents = Vec::new();
    for child in children {
        if let Node::Component(c) = child
            && c.name == "Tab"
        {
            let tab_props = crate::renderer::attributes_to_map(&c.attributes);
            let title = prop_str(&tab_props, "title").unwrap_or("Tab").to_string();
            labels.push(title);
            contents.push(render_children_to_string(&c.children, ctx));
        }
    }
    let storage_key = attrs
        .get("storage_key")
        .or_else(|| attrs.get("storageKey"))
        .cloned();
    let mut map = serde_json::Map::new();
    map.insert("labels".into(), serde_json::json!(labels));
    map.insert("contents".into(), serde_json::json!(contents));
    if let Some(sk) = storage_key {
        map.insert("storage_key".into(), sk);
    }
    serde_json::Value::Object(map)
}

fn build_accordion_props(
    attrs: &HashMap<String, serde_json::Value>,
    children: &[Node],
    ctx: &RenderCtx<'_>,
) -> serde_json::Value {
    let mut items = Vec::new();
    let title = prop_str(attrs, "title").unwrap_or("").to_string();
    let content = render_children_to_string(children, ctx);
    let open = attrs.get("open").and_then(|v| v.as_bool()).unwrap_or(false);
    items.push(serde_json::json!({
        "title": title,
        "content": content,
        "open": open,
    }));
    let multiple = attrs
        .get("multiple")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    serde_json::json!({ "items": items, "multiple": multiple })
}

fn build_code_block_props(
    attrs: &HashMap<String, serde_json::Value>,
    children: &[Node],
    raw_content: &str,
) -> serde_json::Value {
    let language = prop_str(attrs, "language").unwrap_or("").to_string();
    let filename = prop_str(attrs, "filename").unwrap_or("").to_string();
    let line_numbers = attrs
        .get("lineNumbers")
        .or_else(|| attrs.get("line_numbers"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    // Parse highlight attribute ranges (e.g. "1,3-5,8")
    let mut attr_highlights = attrs
        .get("highlight")
        .and_then(|v| v.as_str())
        .map(crate::utils::parse_highlight_ranges)
        .unwrap_or_default();
    // Use raw_content from parser to preserve whitespace/indentation
    let fallback = collect_text(children);
    let code_source = if !raw_content.is_empty() {
        raw_content
    } else {
        &fallback
    };
    let trimmed = code_source.trim_matches('\n');
    // Process comment-based highlight markers (// highlight-next-line, etc.)
    let (code, comment_highlights) = crate::utils::process_highlight_comments(trimmed);
    attr_highlights.extend(comment_highlights);
    let highlight = attr_highlights;
    // Pre-highlight at build time so wasm doesn't need the highlighter
    let raw_html = if !language.is_empty() && oxidoc_highlight::is_supported(&language) {
        oxidoc_highlight::highlight(&code, &language)
    } else {
        crate::utils::html_escape(&code).to_string()
    };
    let code_html = crate::utils::wrap_lines_with_highlights(&raw_html, &highlight);
    serde_json::json!({
        "language": language,
        "code": code,
        "code_html": code_html,
        "filename": filename,
        "line_numbers": line_numbers,
        "highlight_lines": highlight,
    })
}
