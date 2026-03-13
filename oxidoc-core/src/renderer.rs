use rdx_ast::{AttributeValue, Node, Root};
use std::collections::HashMap;
use std::fmt::Write;

use crate::island_props::build_hydration_props;
use crate::static_render::{
    debug_wrap, render_static_accordion, render_static_badge, render_static_callout,
    render_static_card, render_static_card_grid, render_static_code_block, render_static_head,
    render_static_step, render_static_steps, render_static_tab, render_static_tabs,
    render_static_tag, render_static_themed_image, render_static_tooltip,
};
use crate::static_render_landing::{
    render_static_banner, render_static_feature, render_static_feature_grid, render_static_hero,
    render_static_hero_action,
};
use crate::web_component::render_web_component;

/// Rendering context threaded through all render functions.
pub(crate) struct RenderCtx<'a> {
    pub(crate) custom: &'a HashMap<String, String>,
    pub(crate) debug_islands: bool,
}

/// Render a parsed RDX document into an HTML string.
///
/// When `debug_islands` is true, each statically-rendered component gets a
/// visible debug outline so you can identify static vs wasm-hydrated components.
pub fn render_document(
    root: &Root,
    custom_components: &HashMap<String, String>,
    debug_islands: bool,
) -> String {
    let ctx = RenderCtx {
        custom: custom_components,
        debug_islands,
    };
    let mut html = String::with_capacity(4096);
    for node in &root.children {
        render_node(node, &mut html, &ctx);
    }
    html
}

pub(crate) fn render_node(node: &Node, out: &mut String, ctx: &RenderCtx<'_>) {
    match node {
        Node::Heading(h) => {
            let level = h.depth.unwrap_or(1).clamp(1, 6);
            let id = h.id.clone().unwrap_or_else(|| {
                crate::utils::heading_anchor(&crate::utils::extract_plain_text(node))
            });
            let _ = write!(out, "<h{level} id=\"{id}\" class=\"oxidoc-heading\">");
            let _ = write!(
                out,
                "<a href=\"#{id}\" class=\"oxidoc-heading-anchor\" aria-label=\"Link to this section\"><iconify-icon icon=\"lucide:link\" width=\"0.75em\" height=\"0.75em\"></iconify-icon></a>"
            );
            render_children(&h.children, out, ctx);
            let _ = write!(out, "</h{level}>");
        }
        Node::Paragraph(p) => {
            out.push_str("<p>");
            render_children(&p.children, out, ctx);
            out.push_str("</p>");
        }
        Node::Text(t) => {
            push_escaped(&t.value, out);
        }
        Node::Strong(s) => {
            out.push_str("<strong>");
            render_children(&s.children, out, ctx);
            out.push_str("</strong>");
        }
        Node::Emphasis(e) => {
            out.push_str("<em>");
            render_children(&e.children, out, ctx);
            out.push_str("</em>");
        }
        Node::Strikethrough(s) => {
            out.push_str("<del>");
            render_children(&s.children, out, ctx);
            out.push_str("</del>");
        }
        Node::CodeInline(c) => {
            out.push_str("<code>");
            push_escaped(&c.value, out);
            out.push_str("</code>");
        }
        Node::CodeBlock(c) => {
            let lang = c.lang.as_deref().unwrap_or("");
            if lang == "mermaid" {
                // Mermaid diagram — render as <pre class="mermaid"> for mermaid.js
                out.push_str(r#"<pre class="mermaid">"#);
                push_escaped(&c.value, out);
                out.push_str("</pre>");
            } else {
                let lang_attr = if lang.is_empty() {
                    String::new()
                } else {
                    format!(r#" class="language-{}""#, crate::utils::html_escape(lang))
                };
                let _ = write!(out, "<pre><code{lang_attr}>");
                if !lang.is_empty() && oxidoc_highlight::is_supported(lang) {
                    out.push_str(&oxidoc_highlight::highlight(&c.value, lang));
                } else {
                    push_escaped(&c.value, out);
                }
                out.push_str(r#"</code><button class="oxidoc-copy-code" onclick="navigator.clipboard.writeText(this.parentElement.querySelector('code').textContent).then(()=>{this.textContent='Copied!';this.classList.add('copied');setTimeout(()=>{this.textContent='Copy';this.classList.remove('copied')},2000)})">Copy</button></pre>"#);
            }
        }
        Node::List(l) => {
            let tag = if l.ordered == Some(true) { "ol" } else { "ul" };
            let _ = write!(out, "<{tag}>");
            render_children(&l.children, out, ctx);
            let _ = write!(out, "</{tag}>");
        }
        Node::ListItem(li) => {
            if let Some(checked) = li.checked {
                let state = if checked { "checked" } else { "" };
                let _ = write!(
                    out,
                    r#"<li class="task-list-item"><input type="checkbox" disabled {state}> "#
                );
            } else {
                out.push_str("<li>");
            }
            render_children(&li.children, out, ctx);
            out.push_str("</li>");
        }
        Node::Blockquote(bq) => {
            out.push_str("<blockquote>");
            render_children(&bq.children, out, ctx);
            out.push_str("</blockquote>");
        }
        Node::Link(link) => {
            let title_attr = link
                .title
                .as_deref()
                .map(|t| format!(r#" title="{}""#, crate::utils::html_escape(t)))
                .unwrap_or_default();
            let _ = write!(
                out,
                r#"<a href="{}"{title_attr}>"#,
                crate::utils::html_escape(&link.url)
            );
            render_children(&link.children, out, ctx);
            out.push_str("</a>");
        }
        Node::Image(img) => {
            let alt = crate::utils::html_escape(img.alt.as_deref().unwrap_or(""));
            let title_attr = img
                .title
                .as_deref()
                .map(|t| format!(r#" title="{}""#, crate::utils::html_escape(t)))
                .unwrap_or_default();
            let _ = write!(
                out,
                r#"<img src="{}" alt="{alt}"{title_attr} loading="lazy">"#,
                crate::utils::html_escape(&img.url)
            );
        }
        Node::Table(t) => {
            out.push_str("<table>");
            render_children(&t.children, out, ctx);
            out.push_str("</table>");
        }
        Node::TableRow(tr) => {
            out.push_str("<tr>");
            render_children(&tr.children, out, ctx);
            out.push_str("</tr>");
        }
        Node::TableCell(td) => {
            out.push_str("<td>");
            render_children(&td.children, out, ctx);
            out.push_str("</td>");
        }
        Node::ThematicBreak(_) => {
            out.push_str("<hr>");
        }
        Node::Html(h) => {
            render_children(&h.children, out, ctx);
        }
        Node::MathInline(m) => {
            let _ = write!(out, r#"<span class="math math-inline">"#);
            push_escaped(&m.value, out);
            out.push_str("</span>");
        }
        Node::MathDisplay(m) => {
            let _ = write!(out, r#"<div class="math math-display">"#);
            push_escaped(&m.value, out);
            out.push_str("</div>");
        }
        Node::FootnoteDefinition(f) => {
            let _ = write!(out, r#"<div class="footnote" id="fn-{}">"#, f.label);
            render_children(&f.children, out, ctx);
            out.push_str("</div>");
        }
        Node::FootnoteReference(f) => {
            let _ = write!(
                out,
                r##"<sup><a href="#fn-{}" class="footnote-ref">{}</a></sup>"##,
                f.label, f.label
            );
        }
        Node::Component(c) => {
            if let Some(js_src) = ctx.custom.get(&c.name) {
                render_web_component(&c.name, &c.attributes, js_src, out);
            } else {
                render_island_component(
                    &c.name,
                    &c.attributes,
                    &c.children,
                    &c.raw_content,
                    out,
                    ctx,
                );
            }
        }
        Node::Variable(v) => {
            let _ = write!(out, r#"<span data-var="{}">{{}}</span>"#, v.path);
        }
        Node::Error(e) => {
            let _ = write!(
                out,
                r#"<div class="oxidoc-error" data-line="{}">{}</div>"#,
                e.position.start.line, e.message
            );
        }
    }
}

pub(crate) fn render_children(children: &[Node], out: &mut String, ctx: &RenderCtx<'_>) {
    for child in children {
        render_node(child, out, ctx);
    }
}

/// Render a component — built-in components get proper static HTML,
/// unknown components get island placeholders for wasm hydration.
fn render_island_component(
    name: &str,
    attributes: &[rdx_ast::AttributeNode],
    children: &[Node],
    raw_content: &str,
    out: &mut String,
    ctx: &RenderCtx<'_>,
) {
    let props = attributes_to_map(attributes);

    // Components are categorized:
    // - Static-only: purely presentational, no hydration needed (Callout, Card, CardGrid)
    // - Hydration-required: need wasm for interactivity (Tabs, CodeBlock, Accordion)
    // - Unknown: always wrapped as island for wasm hydration
    match name {
        // Static-only components — render as plain HTML, no island wrapper
        "Callout" => render_static_callout(&props, children, out, ctx),
        "CardGrid" => render_static_card_grid(children, out, ctx),
        "Card" => render_static_card(&props, children, out, ctx),
        "Steps" => render_static_steps(children, out, ctx),
        "Step" => render_static_step(&props, children, out, ctx),
        "Badge" => render_static_badge(&props, children, out, ctx),
        "ThemedImage" => render_static_themed_image(&props, out),
        "Tooltip" => render_static_tooltip(&props, children, out, ctx),
        "Banner" => render_static_banner(&props, children, out, ctx),
        "Tag" => render_static_tag(&props, children, out, ctx),
        "Hero" => render_static_hero(&props, children, out, ctx),
        "HeroAction" => render_static_hero_action(&props, children, out, ctx),
        "FeatureGrid" => render_static_feature_grid(children, out, ctx),
        "Feature" => render_static_feature(&props, children, out, ctx),
        "Head" => render_static_head(&props, children, out, ctx),
        // Hydration-required components — SSR inside <oxidoc-island> for wasm to hydrate
        "Tabs" | "Tab" | "Accordion" | "CodeBlock" => {
            let hydration_props = build_hydration_props(name, &props, children, raw_content, ctx);
            let props_json =
                serde_json::to_string(&hydration_props).unwrap_or_else(|_| "{}".into());
            let escaped_props = crate::utils::html_escape(&props_json);

            debug_wrap(name, "hydration", out, ctx.debug_islands, |out| {
                let _ = write!(
                    out,
                    r#"<oxidoc-island data-island-type="{}" data-props="{}">"#,
                    crate::utils::html_escape(&name.to_lowercase()),
                    escaped_props,
                );
                // SSR fallback content (shown before wasm loads)
                match name {
                    "Tabs" => render_static_tabs(children, out, ctx),
                    "Tab" => render_static_tab(&props, children, out, ctx),
                    "Accordion" => render_static_accordion(&props, children, out, ctx),
                    "CodeBlock" => {
                        render_static_code_block(&props, children, raw_content, out, ctx)
                    }
                    _ => unreachable!(),
                }
                out.push_str("</oxidoc-island>");
            });
        }

        // Unknown components — island placeholder for wasm hydration
        _ => {
            let props_json = serde_json::to_string(&props).unwrap_or_else(|_| "{}".into());
            let escaped_props = crate::utils::html_escape(&props_json);
            debug_wrap(name, "hydration", out, ctx.debug_islands, |out| {
                let _ = write!(
                    out,
                    r#"<oxidoc-island data-island-type="{}" data-props="{}">"#,
                    crate::utils::html_escape(&name.to_lowercase()),
                    escaped_props,
                );
                render_children(children, out, ctx);
                out.push_str("</oxidoc-island>");
            });
        }
    }
}

pub(crate) fn attributes_to_map(
    attributes: &[rdx_ast::AttributeNode],
) -> HashMap<String, serde_json::Value> {
    attributes
        .iter()
        .map(|attr| {
            let value = match &attr.value {
                AttributeValue::Null => serde_json::Value::Null,
                AttributeValue::Bool(b) => serde_json::Value::Bool(*b),
                AttributeValue::Number(n) => serde_json::Value::Number(n.clone()),
                AttributeValue::String(s) => serde_json::Value::String(s.clone()),
                AttributeValue::Array(a) => serde_json::Value::Array(a.clone()),
                AttributeValue::Object(o) => serde_json::Value::Object(o.clone()),
                AttributeValue::Variable(v) => {
                    serde_json::Value::String(format!("${{{}}}", v.path))
                }
            };
            (attr.name.clone(), value)
        })
        .collect()
}

fn push_escaped(text: &str, out: &mut String) {
    out.push_str(&crate::utils::html_escape(text));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_html() {
        let mut out = String::new();
        push_escaped("<script>alert('xss')</script>", &mut out);
        assert_eq!(out, "&lt;script&gt;alert('xss')&lt;/script&gt;");
    }

    #[test]
    fn render_empty_document() {
        let root = rdx_parser::parse("");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.is_empty());
    }

    #[test]
    fn render_paragraph_with_text() {
        let root = rdx_parser::parse("Hello, world!");
        let html = render_document(&root, &HashMap::new(), false);
        assert_eq!(html, "<p>Hello, world!</p>");
    }

    #[test]
    fn render_heading_with_anchor() {
        let root = rdx_parser::parse("# Getting Started");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains(r#"<h1 id="getting-started" class="oxidoc-heading">"#));
        assert!(html.contains("Getting Started"));
        assert!(html.contains(r#"class="oxidoc-heading-anchor""#));
        assert!(html.contains("href=\"#getting-started\""));
    }

    #[test]
    fn render_code_block() {
        let root = rdx_parser::parse("```rust\nfn main() {}\n```");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains(r#"class="language-rust""#));
        // Content is syntax-highlighted with tok-* spans
        assert!(html.contains("tok-keyword"));
        assert!(html.contains("main"));
    }

    #[test]
    fn render_emphasis_and_strong() {
        let root = rdx_parser::parse("*italic* and **bold**");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn render_link() {
        let root = rdx_parser::parse("[click here](https://example.com)");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains(r#"<a href="https://example.com">"#));
        assert!(html.contains("click here</a>"));
    }

    #[test]
    fn render_static_component_no_island() {
        // Static-only components (Callout, Card, CardGrid) render plain HTML, no island
        let root = rdx_parser::parse(r#"<Callout kind="warning">Watch out!</Callout>"#);
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains("oxidoc-callout-warning"));
        assert!(html.contains("Watch out!"));
        assert!(!html.contains("oxidoc-island"));
    }

    #[test]
    fn render_hydration_component_as_island() {
        // Hydration-required components (Tabs, CodeBlock, Accordion) get island wrapper
        let root = rdx_parser::parse(r#"<Tabs><Tab title="A">aaa</Tab></Tabs>"#);
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains("oxidoc-island"));
        assert!(html.contains(r#"data-island-type="tabs""#));
        assert!(html.contains("oxidoc-tabs")); // SSR content inside
    }

    #[test]
    fn render_unknown_component_as_island() {
        let root = rdx_parser::parse(r#"<MyWidget foo="bar">content</MyWidget>"#);
        let html = render_document(&root, &HashMap::new(), false);
        assert!(html.contains(r#"data-island-type="mywidget""#));
        assert!(html.contains("data-props="));
    }

    #[test]
    fn render_debug_shows_hydration_check() {
        // Debug mode shows hydration check for interactive components
        let root = rdx_parser::parse(r#"<Tabs><Tab title="X">x</Tab></Tabs>"#);
        let html = render_document(&root, &HashMap::new(), true);
        assert!(html.contains("oxidoc-debug-island"));
        assert!(html.contains("awaiting hydration"));
    }

    #[test]
    fn render_debug_no_wrapper_for_static() {
        // Static components get no debug wrapper (they're correct as-is)
        let root = rdx_parser::parse(r#"<Callout kind="info">test</Callout>"#);
        let html = render_document(&root, &HashMap::new(), true);
        assert!(!html.contains("oxidoc-debug-island"));
        assert!(html.contains("oxidoc-callout-info"));
    }

    #[test]
    fn render_component_as_web_component() {
        let mut custom = HashMap::new();
        custom.insert("PromoBanner".into(), "assets/js/promo.js".into());
        let root = rdx_parser::parse(r#"<PromoBanner variant="dark" />"#);
        let html = render_document(&root, &custom, false);
        assert!(html.contains("<PromoBanner"));
        assert!(html.contains(r#"src="assets/js/promo.js""#));
        assert!(!html.contains("oxidoc-island"));
    }

    #[test]
    fn code_block_language_is_escaped() {
        let root = rdx_parser::parse("```rust\" onclick=\"alert(1)\ncode\n```");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(!html.contains(r#"onclick="alert"#));
        assert!(html.contains("&quot;"));
    }

    #[test]
    fn xss_in_text_is_escaped() {
        let root = rdx_parser::parse("<script>alert('xss')</script>");
        let html = render_document(&root, &HashMap::new(), false);
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn emdash_preserved() {
        let root = rdx_parser::parse("Hello \u{2014} world");
        let html = render_document(&root, &HashMap::new(), false);
        let bytes: Vec<u8> = html.bytes().collect();
        // em dash should be e2 80 94, not double-encoded c3 a2 c2 80 c2 94
        assert!(
            html.contains('\u{2014}'),
            "em dash should be preserved. HTML: {:?}, bytes: {:02x?}",
            html,
            bytes
        );
    }
}
