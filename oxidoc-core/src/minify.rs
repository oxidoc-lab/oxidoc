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
        if in_special_tag {
            // Inside a special tag: look only for the closing tag, pass everything else through
            let closing = format!("</{}>", special_tag_name);
            if bytes[i] == b'<' && html[i..].to_lowercase().starts_with(&closing) {
                // Found closing tag — output it and exit special mode
                let end = i + closing.len();
                result.push_str(&html[i..end]);
                i = end;
                in_special_tag = false;
            } else {
                // Pass through verbatim (handle multi-byte UTF-8)
                // SAFETY: i < bytes.len() (loop guard) guarantees html[i..] is non-empty
                let ch = html[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
        } else if bytes[i] == b'<' {
            // Parse tag
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
            if tag_lower.starts_with("<pre")
                || tag_lower.starts_with("<code")
                || tag_lower.starts_with("<script")
                || tag_lower.starts_with("<style")
            {
                in_special_tag = true;
                if let Some(space_or_close) =
                    tag_lower[1..].find(|c: char| c.is_whitespace() || c == '>')
                {
                    special_tag_name = tag_lower[1..1 + space_or_close].to_string();
                }
            }

            result.push_str(tag_text);
            i = tag_end;
        } else if bytes[i].is_ascii_whitespace() {
            // Collapse whitespace outside special tags
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if !result.is_empty() && !result.ends_with(' ') && i < bytes.len() {
                result.push(' ');
            }
        } else {
            // Handle multi-byte UTF-8 correctly
            // SAFETY: i < bytes.len() (loop guard) guarantees html[i..] is non-empty
            let ch = html[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
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
        assert_eq!(output, input);
    }

    #[test]
    fn minify_preserves_code_content() {
        let input = r#"<code>  x =   1  </code>"#;
        let output = minify_html(input);
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
    fn minify_preserves_script_with_less_than() {
        let input = "<script>for(var i=0;i<10;i++){}</script>";
        let output = minify_html(input);
        assert_eq!(output, input);
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
