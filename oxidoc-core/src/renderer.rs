use rdx_ast::{AttributeValue, Node, Root};
use std::collections::HashMap;
use std::fmt::Write;

/// Render a parsed RDX document into an HTML string.
pub fn render_document(root: &Root, custom_components: &HashMap<String, String>) -> String {
    let mut html = String::with_capacity(4096);
    for node in &root.children {
        render_node(node, &mut html, custom_components);
    }
    html
}

fn render_node(node: &Node, out: &mut String, custom: &HashMap<String, String>) {
    match node {
        Node::Heading(h) => {
            let level = h.depth.unwrap_or(1).clamp(1, 6);
            let id = h.id.clone().unwrap_or_else(|| {
                crate::utils::heading_anchor(&crate::utils::extract_plain_text(node))
            });
            let _ = write!(out, r#"<h{level} id="{id}">"#);
            render_children(&h.children, out, custom);
            let _ = write!(out, "</h{level}>");
        }
        Node::Paragraph(p) => {
            out.push_str("<p>");
            render_children(&p.children, out, custom);
            out.push_str("</p>");
        }
        Node::Text(t) => {
            push_escaped(&t.value, out);
        }
        Node::Strong(s) => {
            out.push_str("<strong>");
            render_children(&s.children, out, custom);
            out.push_str("</strong>");
        }
        Node::Emphasis(e) => {
            out.push_str("<em>");
            render_children(&e.children, out, custom);
            out.push_str("</em>");
        }
        Node::Strikethrough(s) => {
            out.push_str("<del>");
            render_children(&s.children, out, custom);
            out.push_str("</del>");
        }
        Node::CodeInline(c) => {
            out.push_str("<code>");
            push_escaped(&c.value, out);
            out.push_str("</code>");
        }
        Node::CodeBlock(c) => {
            let lang_attr = c
                .lang
                .as_deref()
                .map(|l| format!(r#" class="language-{l}""#))
                .unwrap_or_default();
            let _ = write!(out, "<pre><code{lang_attr}>");
            push_escaped(&c.value, out);
            out.push_str("</code></pre>");
        }
        Node::List(l) => {
            let tag = if l.ordered == Some(true) { "ol" } else { "ul" };
            let _ = write!(out, "<{tag}>");
            render_children(&l.children, out, custom);
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
            render_children(&li.children, out, custom);
            out.push_str("</li>");
        }
        Node::Blockquote(bq) => {
            out.push_str("<blockquote>");
            render_children(&bq.children, out, custom);
            out.push_str("</blockquote>");
        }
        Node::Link(link) => {
            let title_attr = link
                .title
                .as_deref()
                .map(|t| format!(r#" title="{t}""#))
                .unwrap_or_default();
            let _ = write!(out, r#"<a href="{}"{title_attr}>"#, link.url);
            render_children(&link.children, out, custom);
            out.push_str("</a>");
        }
        Node::Image(img) => {
            let alt = img.alt.as_deref().unwrap_or("");
            let title_attr = img
                .title
                .as_deref()
                .map(|t| format!(r#" title="{t}""#))
                .unwrap_or_default();
            let _ = write!(
                out,
                r#"<img src="{}" alt="{alt}"{title_attr} loading="lazy">"#,
                img.url
            );
        }
        Node::Table(t) => {
            out.push_str("<table>");
            render_children(&t.children, out, custom);
            out.push_str("</table>");
        }
        Node::TableRow(tr) => {
            out.push_str("<tr>");
            render_children(&tr.children, out, custom);
            out.push_str("</tr>");
        }
        Node::TableCell(td) => {
            out.push_str("<td>");
            render_children(&td.children, out, custom);
            out.push_str("</td>");
        }
        Node::ThematicBreak(_) => {
            out.push_str("<hr>");
        }
        Node::Html(h) => {
            render_children(&h.children, out, custom);
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
            render_children(&f.children, out, custom);
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
            if let Some(js_src) = custom.get(&c.name) {
                render_web_component(&c.name, &c.attributes, js_src, out);
            } else {
                render_island_component(&c.name, &c.attributes, &c.children, out, custom);
            }
        }
        Node::Variable(v) => {
            // Variables are resolved at a higher level; emit placeholder
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

fn render_children(children: &[Node], out: &mut String, custom: &HashMap<String, String>) {
    for child in children {
        render_node(child, out, custom);
    }
}

/// Render an RDX component as an `<oxidoc-island>` placeholder.
fn render_island_component(
    name: &str,
    attributes: &[rdx_ast::AttributeNode],
    children: &[Node],
    out: &mut String,
    custom: &HashMap<String, String>,
) {
    let props = attributes_to_map(attributes);
    let props_json = serde_json::to_string(&props).unwrap_or_else(|_| "{}".into());
    let _ = write!(
        out,
        r#"<oxidoc-island data-island-type="{}" data-props='{}'>"#,
        name.to_lowercase(),
        props_json,
    );
    // Render children as fallback content (visible before Wasm hydration)
    render_children(children, out, custom);
    out.push_str("</oxidoc-island>");
}

/// Render a Vanilla Web Component passthrough.
fn render_web_component(
    tag: &str,
    attributes: &[rdx_ast::AttributeNode],
    js_src: &str,
    out: &mut String,
) {
    let _ = write!(out, "<{tag}");
    for attr in attributes {
        let val = attribute_value_to_string(&attr.value);
        let _ = write!(out, r#" {}="{val}""#, attr.name);
    }
    let _ = write!(
        out,
        "></{tag}><script src=\"{js_src}\" type=\"module\" async></script>"
    );
}

fn attributes_to_map(attributes: &[rdx_ast::AttributeNode]) -> HashMap<String, serde_json::Value> {
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
        let root = Root {
            node_type: rdx_ast::RootType::Root,
            frontmatter: None,
            children: vec![],
            position: rdx_ast::Position {
                start: rdx_ast::Point {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
                end: rdx_ast::Point {
                    line: 1,
                    column: 1,
                    offset: 0,
                },
            },
        };
        let html = render_document(&root, &HashMap::new());
        assert!(html.is_empty());
    }

    #[test]
    fn render_paragraph_with_text() {
        let root = rdx_parser::parse("Hello, world!");
        let html = render_document(&root, &HashMap::new());
        assert_eq!(html, "<p>Hello, world!</p>");
    }

    #[test]
    fn render_heading_with_anchor() {
        let root = rdx_parser::parse("# Getting Started");
        let html = render_document(&root, &HashMap::new());
        assert!(html.contains(r#"<h1 id="getting-started">"#));
        assert!(html.contains("Getting Started"));
    }

    #[test]
    fn render_code_block() {
        let root = rdx_parser::parse("```rust\nfn main() {}\n```");
        let html = render_document(&root, &HashMap::new());
        assert!(html.contains(r#"class="language-rust""#));
        assert!(html.contains("fn main() {}"));
    }

    #[test]
    fn render_emphasis_and_strong() {
        let root = rdx_parser::parse("*italic* and **bold**");
        let html = render_document(&root, &HashMap::new());
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<strong>bold</strong>"));
    }

    #[test]
    fn render_link() {
        let root = rdx_parser::parse("[click here](https://example.com)");
        let html = render_document(&root, &HashMap::new());
        assert!(html.contains(r#"<a href="https://example.com">"#));
        assert!(html.contains("click here</a>"));
    }

    #[test]
    fn render_component_as_island() {
        let root = rdx_parser::parse(r#"<Callout type="warning">Watch out!</Callout>"#);
        let html = render_document(&root, &HashMap::new());
        assert!(html.contains(r#"data-island-type="callout""#));
        assert!(html.contains("data-props="));
    }

    #[test]
    fn render_component_as_web_component() {
        let mut custom = HashMap::new();
        custom.insert("PromoBanner".into(), "assets/js/promo.js".into());
        let root = rdx_parser::parse(r#"<PromoBanner variant="dark" />"#);
        let html = render_document(&root, &custom);
        assert!(html.contains("<PromoBanner"));
        assert!(html.contains(r#"src="assets/js/promo.js""#));
        assert!(!html.contains("oxidoc-island"));
    }

    #[test]
    fn xss_in_text_is_escaped() {
        let root = rdx_parser::parse("<script>alert('xss')</script>");
        let html = render_document(&root, &HashMap::new());
        assert!(!html.contains("<script>"));
    }
}
