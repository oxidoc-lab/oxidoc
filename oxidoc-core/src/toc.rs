use crate::utils::{extract_plain_text, heading_anchor};
use rdx_ast::{Node, Root};

/// A table-of-contents entry extracted from headings.
#[derive(Debug, Clone)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub anchor: String,
}

/// Extract table of contents from a parsed RDX document.
pub fn extract_toc(root: &Root) -> Vec<TocEntry> {
    let mut entries = Vec::new();
    for node in &root.children {
        collect_headings(node, &mut entries);
    }
    entries
}

fn collect_headings(node: &Node, entries: &mut Vec<TocEntry>) {
    if let Node::Heading(h) = node {
        let level = h.depth.unwrap_or(1).clamp(1, 6);
        let text = extract_plain_text(node);
        let anchor = h.id.clone().unwrap_or_else(|| heading_anchor(&text));
        entries.push(TocEntry {
            level,
            text,
            anchor,
        });
    }
    if let Some(children) = node.children() {
        for child in children {
            collect_headings(child, entries);
        }
    }
}

/// Extract plain text from a node (public convenience for other modules).
pub fn extract_heading_text(node: &Node) -> String {
    extract_plain_text(node)
}

/// Render TOC entries into an HTML `<nav>` element.
pub fn render_toc(entries: &[TocEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut html = String::from(r#"<nav class="oxidoc-toc" aria-label="Table of contents"><ul>"#);
    for entry in entries {
        let indent_class = format!("toc-level-{}", entry.level);
        html.push_str(&format!(
            r##"<li class="{indent_class}"><a href="#{}">{}</a></li>"##,
            entry.anchor, entry.text,
        ));
    }
    html.push_str("</ul></nav>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_toc_from_document() {
        let root = rdx_parser::parse("# Title\n\nSome text\n\n## Section One\n\n### Subsection");
        let toc = extract_toc(&root);
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "Title");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[1].text, "Section One");
        assert_eq!(toc[2].level, 3);
    }

    #[test]
    fn empty_document_empty_toc() {
        let root = rdx_parser::parse("Just a paragraph, no headings.");
        let toc = extract_toc(&root);
        assert!(toc.is_empty());
    }

    #[test]
    fn render_toc_html() {
        let entries = vec![
            TocEntry {
                level: 1,
                text: "Intro".into(),
                anchor: "intro".into(),
            },
            TocEntry {
                level: 2,
                text: "Setup".into(),
                anchor: "setup".into(),
            },
        ];
        let html = render_toc(&entries);
        assert!(html.contains(r#"class="toc-level-1""#));
        assert!(html.contains(r##"href="#intro""##));
        assert!(html.contains(r#"class="toc-level-2""#));
    }

    #[test]
    fn render_toc_empty() {
        assert!(render_toc(&[]).is_empty());
    }
}
