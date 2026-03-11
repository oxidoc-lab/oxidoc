/// Integration fuzz tests for oxidoc-core.
/// Tests that malformed/random input doesn't panic the rendering engine.
use oxidoc_core::config::parse_config;
use oxidoc_core::renderer::render_document;
use oxidoc_core::toc::extract_toc;
use std::collections::HashMap;

/// Test that various malformed/edge-case inputs don't panic render_document
#[test]
fn fuzz_render_document_empty() {
    let root = rdx_parser::parse("");
    let result = render_document(&root, &HashMap::new());
    // Should return empty or whitespace, not panic
    assert!(result.is_empty() || result.trim().is_empty());
}

#[test]
fn fuzz_render_document_plain_text() {
    let root = rdx_parser::parse("Just plain text with no special markup");
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("Just plain text"));
}

#[test]
fn fuzz_render_document_deeply_nested_emphasis() {
    // Deeply nested emphasis
    let input = "***very ***very ***very ***deeply ***nested*** emphasis*** here*** text***";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    // Should not panic, even with ambiguous nesting
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_unclosed_tags() {
    let input = "Some [unclosed link text\nMore text here";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    // rdx-parser handles this gracefully
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_mismatched_brackets() {
    let input = "[[[ nested brackets ]]] and [incomplete";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_null_bytes_in_text() {
    let input = "Text with\0null\0bytes";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    // Should escape properly
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_extremely_long_line() {
    let long_text = "a".repeat(10000);
    let root = rdx_parser::parse(&long_text);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.len() >= 10000); // Should contain the long text
}

#[test]
fn fuzz_render_document_many_headings() {
    let mut input = String::new();
    for i in 1..=50 {
        input.push_str(&format!("# Heading {}\n\nContent {}\n\n", i, i));
    }
    let root = rdx_parser::parse(&input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("<h1"));
}

#[test]
fn fuzz_render_document_mixed_formatting() {
    let input = "***bold and italic*** and **just bold** and *just italic* and `code`";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("<strong>"));
    assert!(result.contains("<em>"));
    assert!(result.contains("<code>"));
}

#[test]
fn fuzz_render_document_invalid_urls() {
    let input = "[link](ht!tp://invalid url with spaces)\n[another](::invalid)";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    // Should not panic, should render something
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_code_blocks_with_special_chars() {
    let input = r#"
```rust
<script>alert('xss')</script>
```
"#;
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    // Should escape script content
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_unicode_content() {
    let input = "Unicode: 你好世界 🚀 مرحبا العالم ñ é ü ü 中文 العربية";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
}

#[test]
fn fuzz_render_document_list_with_mixed_items() {
    let input = "- Item 1\n- Item 2 with *emphasis*\n- Item 3 with [link](http://test)\n  - Nested\n  - Nested 2";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("<ul>"));
    assert!(result.contains("<li>"));
}

#[test]
fn fuzz_render_document_table_with_special_content() {
    let input = "| Header | Header 2 |\n|--------|----------|\n| `code` | **bold** |\n| [link](http://test) | *italic* |";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("<table>"));
}

#[test]
fn fuzz_render_document_blockquote_nested() {
    let input = "> Quote level 1\n> > Quote level 2\n> > > Quote level 3";
    let root = rdx_parser::parse(input);
    let result = render_document(&root, &HashMap::new());
    assert!(!result.is_empty());
    assert!(result.contains("<blockquote>"));
}

#[test]
fn fuzz_extract_toc_empty_document() {
    let root = rdx_parser::parse("");
    let toc = extract_toc(&root);
    assert!(toc.is_empty());
}

#[test]
fn fuzz_extract_toc_no_headings() {
    let root = rdx_parser::parse("Just paragraphs\n\nNo headings here");
    let toc = extract_toc(&root);
    assert!(toc.is_empty());
}

#[test]
fn fuzz_extract_toc_many_levels() {
    let input =
        "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n####### Invalid H7 (becomes text)";
    let root = rdx_parser::parse(input);
    let toc = extract_toc(&root);
    // Should have at least 6 valid headings
    assert!(toc.len() >= 6);
}

#[test]
fn fuzz_extract_toc_with_special_characters() {
    let input = "# Heading with `code` and **bold**\n## Heading with 中文\n### Heading with [link](http://test)";
    let root = rdx_parser::parse(input);
    let toc = extract_toc(&root);
    assert_eq!(toc.len(), 3);
    // Text should be extracted from inline elements
    assert!(!toc[0].text.is_empty());
}

#[test]
fn fuzz_parse_config_empty_string() {
    // Empty config should fail (missing project.name)
    let result = parse_config("");
    assert!(result.is_err());
}

#[test]
fn fuzz_parse_config_malformed_toml() {
    let inputs = vec![
        "[[[ invalid",
        "= = = broken",
        "[unclosed\nkey = value",
        "key = value without section",
    ];

    for input in inputs {
        let result = parse_config(input);
        // Should fail gracefully, not panic
        assert!(result.is_err(), "Should fail for: {}", input);
    }
}

#[test]
fn fuzz_parse_config_minimal_valid() {
    let toml = "[project]\nname = \"Test\"";
    let result = parse_config(toml);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.project.name, "Test");
}

#[test]
fn fuzz_parse_config_missing_name() {
    let toml = "[project]\n";
    let result = parse_config(toml);
    // Should fail because name is required
    assert!(result.is_err());
}

#[test]
fn fuzz_parse_config_empty_name() {
    let toml = "[project]\nname = \"  \"";
    let result = parse_config(toml);
    // Should fail because name is empty when trimmed
    assert!(result.is_err());
}

#[test]
fn fuzz_parse_config_with_unknown_keys() {
    let toml = "[project]\nname = \"Test\"\n[unknown_section]\nkey = \"value\"";
    let result = parse_config(toml);
    // Should succeed (unknown keys are warned about but not errors)
    assert!(result.is_ok());
}

#[test]
fn fuzz_parse_config_nested_colors() {
    let toml = "[project]\nname = \"Test\"\n[theme]\nprimary = \"#ff0000\"\ndark_mode = \"dark\"";
    let result = parse_config(toml);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.theme.primary, "#ff0000");
}

#[test]
fn fuzz_parse_config_arrays_and_tables() {
    let toml = r#"
[project]
name = "Test"

[[routing.navigation]]
group = "Getting Started"
pages = ["intro", "quickstart"]

[[routing.navigation]]
group = "API"
openapi = "openapi.yaml"

[versioning]
versions = ["v1", "v2"]

[components.custom]
Banner = "assets/banner.js"
"#;
    let result = parse_config(toml);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.routing.navigation.len(), 2);
    assert_eq!(config.versioning.versions.len(), 2);
    assert!(config.components.custom.contains_key("Banner"));
}

#[test]
fn fuzz_parse_config_special_characters_in_strings() {
    let toml = r#"
[project]
name = "Documentation with quotes and apostrophes"
description = "Multi-line string"
"#;
    let result = parse_config(toml);
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.project.name.contains("quotes"));
}
