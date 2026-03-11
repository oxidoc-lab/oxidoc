/// Minify HTML by collapsing whitespace between tags while preserving content in special tags.
/// This is a simple implementation that collapses consecutive whitespace to single spaces,
/// but preserves all content within special tags like <pre>, <code>, <script>, and <style>.
pub fn minify_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_special_tag = false;
    let mut special_tag_name = String::new();
    let mut i = 0;
    let bytes = html.as_bytes();

    while i < bytes.len() {
        // Check for opening tag
        if bytes[i] == b'<' {
            let tag_start = i;
            i += 1;
            let mut tag_end = i;
            while tag_end < bytes.len() && bytes[tag_end] != b'>' {
                tag_end += 1;
            }
            if tag_end < bytes.len() {
                tag_end += 1; // Include the '>'
            }

            let tag_text = std::str::from_utf8(&bytes[tag_start..tag_end]).unwrap_or("");
            let tag_lower = tag_text.to_lowercase();

            // Check if entering a special tag
            if !in_special_tag
                && (tag_lower.starts_with("<pre")
                    || tag_lower.starts_with("<code")
                    || tag_lower.starts_with("<script")
                    || tag_lower.starts_with("<style"))
            {
                in_special_tag = true;
                // Extract tag name
                if let Some(space_or_close) =
                    tag_lower[1..].find(|c: char| c.is_whitespace() || c == '>')
                {
                    special_tag_name = tag_lower[1..1 + space_or_close].to_string();
                }
            }

            // Check if exiting a special tag
            if in_special_tag {
                let closing = format!("</{}>", special_tag_name);
                if tag_lower.starts_with(&closing.to_lowercase()) {
                    in_special_tag = false;
                }
            }

            result.push_str(tag_text);
            i = tag_end;
        } else if !in_special_tag && bytes[i].is_ascii_whitespace() {
            // Collapse whitespace outside special tags
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if !result.is_empty() && !result.ends_with(' ') && i < bytes.len() {
                result.push(' ');
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    // Clean up trailing whitespace
    while result.ends_with(|c: char| c.is_whitespace()) {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_collapses_whitespace() {
        let input = "<div>  Hello   World  </div>";
        let output = minify_html(input);
        assert!(!output.contains("   "));
    }

    #[test]
    fn minify_preserves_pre_content() {
        let input = r#"<pre>  code   with   spaces  </pre>"#;
        let output = minify_html(input);
        // Should preserve exact content in pre tags
        assert_eq!(output, input);
    }

    #[test]
    fn minify_preserves_code_content() {
        let input = r#"<code>  x =   1  </code>"#;
        let output = minify_html(input);
        // Should preserve exact content in code tags
        assert_eq!(output, input);
    }

    #[test]
    fn minify_removes_newlines() {
        let input = "<div>\n  <p>Hello</p>\n</div>";
        let output = minify_html(input);
        assert!(!output.contains('\n'));
    }

    #[test]
    fn minify_preserves_script_content() {
        let input = "<script>\n  const x = 1;\n  console.log(x);\n</script>";
        let output = minify_html(input);
        assert!(output.contains("const x = 1;"));
    }

    #[test]
    fn minify_preserves_style_content() {
        let input = "<style>  body { margin: 0; }  </style>";
        let output = minify_html(input);
        assert_eq!(output, input);
    }

    #[test]
    fn minify_handles_nested_tags() {
        let input = "<div>   <span>   text   </span>   </div>";
        let output = minify_html(input);
        assert!(output.contains("<div>"));
        assert!(output.contains("</div>"));
    }

    #[test]
    fn minify_handles_empty_tags() {
        let input = "<br>  <img>";
        let output = minify_html(input);
        assert!(output.contains("<br>"));
        assert!(output.contains("<img>"));
    }
}
