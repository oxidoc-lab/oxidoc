//! Static HTML rendering for built-in components (Callout, Tabs, Card, etc.).
//!
//! These functions produce semantic HTML that works without JavaScript,
//! as opposed to island placeholders that require Wasm hydration.

use rdx_ast::Node;
use std::collections::HashMap;
use std::fmt::Write;

use crate::renderer::{RenderCtx, attributes_to_map, render_children};

/// Extract a string value from a props map, returning `None` if the key is
/// missing or the value is not a string.
pub(crate) fn prop_str<'a>(
    props: &'a HashMap<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    props.get(key).and_then(|v| v.as_str())
}

/// Emit a debug outline wrapper when `debug_islands` is enabled.
///
/// The outline colour distinguishes static renders (amber) from Wasm-hydrated
/// islands (green), making it easy to identify rendering mode during development.
pub(crate) fn debug_wrap(
    name: &str,
    mode: &str,
    out: &mut String,
    debug: bool,
    f: impl FnOnce(&mut String),
) {
    if debug {
        let color = if mode == "static" {
            "#f59e0b"
        } else {
            "#10b981"
        };
        let _ = write!(
            out,
            r#"<div class="oxidoc-debug-island" data-component="{name}" data-render="{mode}" style="outline:2px dashed {color};outline-offset:2px;position:relative">"#,
        );
        let _ = write!(
            out,
            r#"<span style="position:absolute;top:-10px;right:4px;font-size:10px;background:{color};color:#fff;padding:0 4px;border-radius:2px;z-index:99">{name} ({mode})</span>"#,
        );
    }
    f(out);
    if debug {
        out.push_str("</div>");
    }
}

pub(crate) fn render_static_callout(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let kind = prop_str(props, "kind").unwrap_or("info");
    let title = prop_str(props, "title");
    let _ = write!(
        out,
        r#"<div class="oxidoc-callout oxidoc-callout-{kind}" role="note">"#,
    );
    if let Some(t) = title {
        let _ = write!(
            out,
            r#"<p class="oxidoc-callout-title"><strong>{}</strong></p>"#,
            crate::utils::html_escape(t)
        );
    }
    out.push_str(r#"<div class="oxidoc-callout-content">"#);
    render_children(children, out, ctx);
    out.push_str("</div></div>");
}

pub(crate) fn render_static_card_grid(children: &[Node], out: &mut String, ctx: &RenderCtx<'_>) {
    out.push_str(r#"<div class="oxidoc-card-grid">"#);
    render_children(children, out, ctx);
    out.push_str("</div>");
}

pub(crate) fn render_static_card(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    let href = prop_str(props, "href");
    if let Some(href) = href {
        let _ = write!(
            out,
            r#"<a href="{}" class="oxidoc-card">"#,
            crate::utils::html_escape(href)
        );
    } else {
        out.push_str(r#"<div class="oxidoc-card">"#);
    }
    let _ = write!(
        out,
        r#"<h3 class="oxidoc-card-title">{}</h3>"#,
        crate::utils::html_escape(title)
    );
    out.push_str(r#"<div class="oxidoc-card-body">"#);
    render_children(children, out, ctx);
    out.push_str("</div>");
    if href.is_some() {
        out.push_str("</a>");
    } else {
        out.push_str("</div>");
    }
}

pub(crate) fn render_static_tabs(children: &[Node], out: &mut String, ctx: &RenderCtx<'_>) {
    out.push_str(r#"<div class="oxidoc-tabs">"#);
    out.push_str(r#"<div class="oxidoc-tabs-nav" role="tablist">"#);
    for (i, child) in children.iter().enumerate() {
        if let Node::Component(c) = child
            && c.name == "Tab"
        {
            let tab_props = attributes_to_map(&c.attributes);
            let title = prop_str(&tab_props, "title").unwrap_or("Tab");
            let active = if i == 0 { r#" class="active""# } else { "" };
            let _ = write!(
                out,
                r#"<button role="tab"{active}>{}</button>"#,
                crate::utils::html_escape(title)
            );
        }
    }
    out.push_str("</div>");
    for (i, child) in children.iter().enumerate() {
        if let Node::Component(c) = child
            && c.name == "Tab"
        {
            let hidden = if i == 0 { "" } else { r#" hidden"# };
            let _ = write!(
                out,
                r#"<div class="oxidoc-tab-panel" role="tabpanel"{hidden}>"#
            );
            render_children(&c.children, out, ctx);
            out.push_str("</div>");
        }
    }
    out.push_str("</div>");
}

pub(crate) fn render_static_tab(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    if !title.is_empty() {
        let _ = write!(out, "<strong>{}</strong>", crate::utils::html_escape(title));
    }
    render_children(children, out, ctx);
}

pub(crate) fn render_static_accordion(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let title = prop_str(props, "title").unwrap_or("");
    out.push_str(r#"<details class="oxidoc-accordion">"#);
    let _ = write!(
        out,
        r#"<summary>{}</summary>"#,
        crate::utils::html_escape(title)
    );
    out.push_str(r#"<div class="oxidoc-accordion-content">"#);
    render_children(children, out, ctx);
    out.push_str("</div></details>");
}

pub(crate) fn render_static_code_block(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let language = prop_str(props, "language").unwrap_or("");
    let filename = prop_str(props, "filename");
    if let Some(fname) = filename {
        let _ = write!(
            out,
            r#"<div class="oxidoc-code-filename">{}</div>"#,
            crate::utils::html_escape(fname)
        );
    }
    let _ = write!(
        out,
        r#"<pre class="oxidoc-code"><code class="language-{language}">"#
    );
    render_children(children, out, ctx);
    out.push_str("</code></pre>");
}
