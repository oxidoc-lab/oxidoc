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
        Node::Text(t) => text.push_str(&t.value),
        Node::CodeInline(t) => text.push_str(&t.value),
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

pub fn extract_plain_text_from_nodes(nodes: &[Node]) -> String {
    let mut text = String::new();
    for node in nodes {
        text.push_str(&extract_plain_text(node));
    }
    text
}

/// Parse a highlight range string like "1,2,3,5-10,15" into a set of line numbers.
pub fn parse_highlight_ranges(s: &str) -> Vec<usize> {
    let mut lines = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((start, end)) = part.split_once('-') {
            if let (Ok(s), Ok(e)) = (start.trim().parse::<usize>(), end.trim().parse::<usize>()) {
                for n in s..=e {
                    lines.push(n);
                }
            }
        } else if let Ok(n) = part.parse::<usize>() {
            lines.push(n);
        }
    }
    lines
}

/// Comment markers for highlight directives.
/// These are stripped from the output and used to mark lines for highlighting.
const HIGHLIGHT_NEXT: &[&str] = &[
    "// highlight-next-line",
    "# highlight-next-line",
    "<!-- highlight-next-line -->",
];
const HIGHLIGHT_START: &[&str] = &[
    "// highlight-start",
    "# highlight-start",
    "<!-- highlight-start -->",
];
const HIGHLIGHT_END: &[&str] = &[
    "// highlight-end",
    "# highlight-end",
    "<!-- highlight-end -->",
];

/// Process code to extract comment-based highlight markers.
/// Returns (cleaned code with markers stripped, set of 1-based line numbers to highlight).
pub fn process_highlight_comments(code: &str) -> (String, Vec<usize>) {
    let mut output_lines = Vec::new();
    let mut highlight_lines = Vec::new();
    let mut highlight_next = false;
    let mut in_highlight_block = false;

    for line in code.lines() {
        let trimmed = line.trim();

        if HIGHLIGHT_NEXT.contains(&trimmed) {
            highlight_next = true;
            continue; // strip marker line
        }
        if HIGHLIGHT_START.contains(&trimmed) {
            in_highlight_block = true;
            continue; // strip marker line
        }
        if HIGHLIGHT_END.contains(&trimmed) {
            in_highlight_block = false;
            continue; // strip marker line
        }

        output_lines.push(line);
        let line_num = output_lines.len(); // 1-based

        if highlight_next || in_highlight_block {
            highlight_lines.push(line_num);
            highlight_next = false;
        }
    }

    (output_lines.join("\n"), highlight_lines)
}

/// Apply line highlighting to already-highlighted HTML.
/// Wraps each line in a span with `oxidoc-line` class, adding `highlighted` for specified lines.
pub fn wrap_lines_with_highlights(html: &str, highlight_lines: &[usize]) -> String {
    if highlight_lines.is_empty() {
        return html.to_string();
    }
    html.split('\n')
        .enumerate()
        .map(|(i, line)| {
            let num = i + 1;
            let class = if highlight_lines.contains(&num) {
                "oxidoc-line highlighted"
            } else {
                "oxidoc-line"
            };
            format!(r#"<span class="{class}">{line}</span>"#)
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Get git last-modified date and author for a file.
/// Returns (formatted_date, author_name) or None if git info unavailable.
pub fn git_file_meta(file_path: &std::path::Path) -> Option<(String, String)> {
    let dir = file_path.parent()?;
    let output = std::process::Command::new("git")
        .args(["log", "-1", "--format=%aI\t%aN", "--"])
        .arg(file_path)
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.trim();
    if line.is_empty() {
        return None;
    }
    let (date_str, author) = line.split_once('\t')?;
    // Parse ISO date and format nicely
    let date = chrono::DateTime::parse_from_rfc3339(date_str).ok()?;
    let formatted = date.format("%b %-d, %Y").to_string();
    Some((formatted, author.to_string()))
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

    #[test]
    fn parse_highlight_ranges_basic() {
        assert_eq!(parse_highlight_ranges("2"), vec![2]);
        assert_eq!(parse_highlight_ranges("1,3,5"), vec![1, 3, 5]);
        assert_eq!(parse_highlight_ranges("2-5"), vec![2, 3, 4, 5]);
        assert_eq!(parse_highlight_ranges("1,3-5,8"), vec![1, 3, 4, 5, 8]);
        assert_eq!(parse_highlight_ranges(""), Vec::<usize>::new());
        assert_eq!(parse_highlight_ranges("1, 3 - 5, 8"), vec![1, 3, 4, 5, 8]);
    }

    #[test]
    fn highlight_comments_next_line() {
        let code = "line1\n// highlight-next-line\nline2\nline3";
        let (cleaned, hl) = process_highlight_comments(code);
        assert_eq!(cleaned, "line1\nline2\nline3");
        assert_eq!(hl, vec![2]); // line2 is now line 2 after stripping
    }

    #[test]
    fn highlight_comments_block() {
        let code = "line1\n// highlight-start\nline2\nline3\n// highlight-end\nline4";
        let (cleaned, hl) = process_highlight_comments(code);
        assert_eq!(cleaned, "line1\nline2\nline3\nline4");
        assert_eq!(hl, vec![2, 3]);
    }

    #[test]
    fn highlight_comments_hash() {
        let code = "line1\n# highlight-next-line\nline2";
        let (cleaned, hl) = process_highlight_comments(code);
        assert_eq!(cleaned, "line1\nline2");
        assert_eq!(hl, vec![2]);
    }

    #[test]
    fn highlight_comments_html() {
        let code = "line1\n<!-- highlight-next-line -->\nline2";
        let (cleaned, hl) = process_highlight_comments(code);
        assert_eq!(cleaned, "line1\nline2");
        assert_eq!(hl, vec![2]);
    }
}
