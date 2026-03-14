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
    if debug && mode == "hydration" {
        // Hydration-required component: render with a debug wrapper that checks
        // if wasm actually hydrated it. If not, show an error after timeout.
        let id = format!(
            "oxidoc-dbg-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let _ = write!(
            out,
            r#"<div id="{id}" class="oxidoc-debug-island" data-component="{name}" data-render="{mode}" style="outline:2px dashed #ef4444;outline-offset:2px;position:relative">"#,
        );
        let _ = write!(
            out,
            r#"<span class="oxidoc-debug-label" style="position:absolute;top:-10px;right:4px;font-size:10px;background:#ef4444;color:#fff;padding:0 4px;border-radius:2px;z-index:99">{name} (awaiting hydration)</span>"#,
        );
        f(out);
        // Watch for data-hydrated attribute via MutationObserver for instant feedback.
        // Falls back to 5s timeout to show error if hydration never happens.
        let _ = write!(
            out,
            r##"<script>(function(){{var d=document.getElementById("{id}");if(!d)return;var i=d.querySelector("oxidoc-island");if(!i)return;function ok(){{d.style.outline="2px solid #10b981";var l=d.querySelector(".oxidoc-debug-label");if(l){{l.style.background="#10b981";l.textContent="{name} (hydrated)"}}}}if(i.getAttribute("data-hydrated")==="true"){{ok();return}}var o=new MutationObserver(function(m){{if(i.getAttribute("data-hydrated")==="true"){{ok();o.disconnect()}}}});o.observe(i,{{attributes:true,attributeFilter:["data-hydrated"]}});setTimeout(function(){{o.disconnect();if(i.getAttribute("data-hydrated")!=="true"){{d.style.outline="2px dashed #ef4444";var l=d.querySelector(".oxidoc-debug-label");if(l){{l.style.background="#ef4444";l.textContent="{name} (failed)"}}}}}},5000)}})()</script>"##,
        );
        out.push_str("</div>");
    } else {
        // Static-only or debug off: just render content directly, no wrapper
        f(out);
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
    out.push_str(r#"<div class="oxidoc-accordion-content"><div>"#);
    render_children(children, out, ctx);
    out.push_str("</div></div></details>");
}

pub(crate) fn render_static_steps(children: &[Node], out: &mut String, ctx: &RenderCtx<'_>) {
    out.push_str(r#"<div class="oxidoc-steps">"#);
    let mut step_num = 0u32;
    for child in children {
        if let Node::Component(c) = child
            && c.name == "Step"
        {
            step_num += 1;
            let step_props = attributes_to_map(&c.attributes);
            let title = prop_str(&step_props, "title").unwrap_or("");
            out.push_str(r#"<div class="oxidoc-step">"#);
            let _ = write!(
                out,
                r#"<div class="oxidoc-step-indicator"><span class="oxidoc-step-number">{step_num}</span></div>"#,
            );
            out.push_str(r#"<div class="oxidoc-step-content">"#);
            if !title.is_empty() {
                let _ = write!(
                    out,
                    r#"<h3 class="oxidoc-step-title">{}</h3>"#,
                    crate::utils::html_escape(title)
                );
            }
            out.push_str(r#"<div class="oxidoc-step-body">"#);
            render_children(&c.children, out, ctx);
            out.push_str("</div></div></div>");
        }
    }
    out.push_str("</div>");
}

pub(crate) fn render_static_step(
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

pub(crate) fn render_static_themed_image(
    props: &HashMap<String, serde_json::Value>,
    out: &mut String,
) {
    let light = prop_str(props, "light").unwrap_or("");
    let dark = prop_str(props, "dark").unwrap_or(light);
    let alt = prop_str(props, "alt").unwrap_or("");
    let width = prop_str(props, "width").unwrap_or("");
    let height = prop_str(props, "height").unwrap_or("");
    let width_attr = if width.is_empty() {
        String::new()
    } else {
        format!(r#" width="{}""#, crate::utils::html_escape(width))
    };
    let height_attr = if height.is_empty() {
        String::new()
    } else {
        format!(r#" height="{}""#, crate::utils::html_escape(height))
    };
    let _ = write!(
        out,
        r#"<picture class="oxidoc-themed-image"><source media="(prefers-color-scheme: dark)" srcset="{dark_src}"><img src="{light_src}" alt="{alt_esc}"{width_attr}{height_attr} loading="lazy"></picture>"#,
        dark_src = crate::utils::html_escape(dark),
        light_src = crate::utils::html_escape(light),
        alt_esc = crate::utils::html_escape(alt),
    );
}

pub(crate) fn render_static_head(
    _props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    _ctx: &RenderCtx<'_>,
) {
    // Head component collects raw HTML children into a special marker
    // that the template engine will move into <head>
    out.push_str("<!--oxidoc-head-start-->");
    for child in children {
        if let Node::Html(h) = child {
            for c in &h.children {
                if let Node::Text(t) = c {
                    out.push_str(&t.value);
                }
            }
        } else if let Node::Text(t) = child {
            out.push_str(&t.value);
        }
    }
    out.push_str("<!--oxidoc-head-end-->");
}

pub(crate) fn render_static_tag(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let kind = prop_str(props, "variant")
        .or_else(|| prop_str(props, "kind"))
        .unwrap_or("info");
    let text = prop_str(props, "text").unwrap_or("");
    let _ = write!(out, r#"<span class="oxidoc-tag oxidoc-tag-{kind}">"#,);
    if !text.is_empty() {
        out.push_str(&crate::utils::html_escape(text));
    } else {
        render_children(children, out, ctx);
    }
    out.push_str("</span>");
}

pub(crate) fn render_static_tooltip(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let tip = prop_str(props, "text")
        .or_else(|| prop_str(props, "tip"))
        .unwrap_or("");
    let _ = write!(
        out,
        r#"<span class="oxidoc-tooltip" tabindex="0"><span class="oxidoc-tooltip-content">"#,
    );
    render_children(children, out, ctx);
    let _ = write!(
        out,
        r#"</span><span class="oxidoc-tooltip-text" role="tooltip">{}</span></span>"#,
        crate::utils::html_escape(tip)
    );
}

pub(crate) fn render_static_badge(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let kind = prop_str(props, "variant")
        .or_else(|| prop_str(props, "kind"))
        .unwrap_or("info");
    let text = prop_str(props, "text").unwrap_or("");
    let outline = props
        .get("outline")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let outline_class = if outline { " oxidoc-badge-outline" } else { "" };
    let _ = write!(
        out,
        r#"<span class="oxidoc-badge oxidoc-badge-{kind}{outline_class}">"#,
    );
    if !text.is_empty() {
        out.push_str(&crate::utils::html_escape(text));
    } else {
        render_children(children, out, ctx);
    }
    out.push_str("</span>");
}

pub(crate) fn render_static_code_block(
    props: &HashMap<String, serde_json::Value>,
    children: &[Node],
    raw_content: &str,
    out: &mut String,
    _ctx: &RenderCtx<'_>,
) {
    let language = prop_str(props, "language").unwrap_or("");
    let filename = prop_str(props, "filename");
    let highlight_attr = prop_str(props, "highlight").unwrap_or("");
    let raw_code = if !raw_content.is_empty() {
        raw_content.to_string()
    } else {
        crate::utils::extract_plain_text_from_nodes(children)
    };
    let trimmed = raw_code.trim_matches('\n');

    // Process comment-based highlight markers
    let (code, comment_highlights) = crate::utils::process_highlight_comments(trimmed);
    let mut hl_lines = crate::utils::parse_highlight_ranges(highlight_attr);
    hl_lines.extend(comment_highlights);

    let raw_html = if !language.is_empty() && oxidoc_highlight::is_supported(language) {
        oxidoc_highlight::highlight(&code, language)
    } else {
        crate::utils::html_escape(&code).to_string()
    };

    let highlighted = crate::utils::wrap_lines_with_highlights(&raw_html, &hl_lines);

    let copy_btn = r#"<button class="oxidoc-copy-code" onclick="navigator.clipboard.writeText(this.parentElement.querySelector('code').textContent).then(()=>{this.textContent='Copied!';this.classList.add('copied');setTimeout(()=>{this.textContent='Copy';this.classList.remove('copied')},2000)})">Copy</button>"#;

    if let Some(fname) = filename {
        // Wrapped CodeBlock with header
        out.push_str(r#"<div class="oxidoc-codeblock">"#);
        let _ = write!(
            out,
            r#"<div class="oxidoc-codeblock-header"><span>{}</span><span>{}</span></div>"#,
            crate::utils::html_escape(fname),
            crate::utils::html_escape(language),
        );
        let _ = write!(
            out,
            r#"<div class="oxidoc-codeblock-body"><pre><code class="language-{}">{}</code>{}</pre></div></div>"#,
            crate::utils::html_escape(language),
            highlighted,
            copy_btn,
        );
    } else {
        // Plain code block (same as markdown fenced code)
        let _ = write!(
            out,
            r#"<pre><code class="language-{}">{}</code>{}</pre>"#,
            crate::utils::html_escape(language),
            highlighted,
            copy_btn,
        );
    }
}
