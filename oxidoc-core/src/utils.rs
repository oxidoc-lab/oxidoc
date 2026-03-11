use rdx_ast::Node;

/// Escape text for safe HTML insertion.
pub fn html_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Generate a URL-safe anchor from heading text.
pub fn heading_anchor(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Extract plain text content recursively from an AST node.
pub fn extract_plain_text(node: &Node) -> String {
    let mut text = String::new();
    match node {
        Node::Text(t) | Node::CodeInline(t) => text.push_str(&t.value),
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    text.push_str(&extract_plain_text(child));
                }
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_anchor_basic() {
        assert_eq!(heading_anchor("Hello World"), "hello-world");
        assert_eq!(heading_anchor("Getting Started!"), "getting-started");
        assert_eq!(heading_anchor("API v2.0"), "api-v2-0");
    }

    #[test]
    fn heading_anchor_empty() {
        assert_eq!(heading_anchor(""), "");
        assert_eq!(heading_anchor("---"), "");
    }

    #[test]
    fn extract_text_from_paragraph() {
        let root = rdx_parser::parse("Hello **world** and `code`");
        let text = extract_plain_text(&root.children[0]);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
        assert!(text.contains("code"));
    }
}
