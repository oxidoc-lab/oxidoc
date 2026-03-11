use rdx_ast::Node;

/// Escape text for safe HTML insertion.
pub fn html_escape(text: &str) -> String {
    escape_text(text, false)
}

/// Escape text for safe XML insertion (includes single quote escaping).
pub fn xml_escape(text: &str) -> String {
    escape_text(text, true)
}

fn escape_text(text: &str, escape_single_quotes: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' if escape_single_quotes => out.push_str("&apos;"),
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
    fn html_escape_special_chars() {
        assert_eq!(html_escape("A & B"), "A &amp; B");
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(html_escape(r#"a "b""#), "a &quot;b&quot;");
        assert_eq!(html_escape("Hello World"), "Hello World");
    }

    #[test]
    fn xml_escape_includes_single_quotes() {
        assert_eq!(xml_escape("A & B"), "A &amp; B");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(
            xml_escape(r#"Quote "text" and 'single'"#),
            r#"Quote &quot;text&quot; and &apos;single&apos;"#
        );
        assert_eq!(xml_escape("Hello World"), "Hello World");
    }

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
