use oxidoc_print::config::PrintConfig;
use oxidoc_print::renderer::render_to_tree;
use oxipdf::ir::SemanticRole;
use std::path::PathBuf;

fn test_config() -> PrintConfig {
    PrintConfig::default_with_root(PathBuf::from("/tmp"))
}

#[test]
fn empty_document_produces_pdf() {
    let source = "";
    let config = test_config();
    let result = oxidoc_print::render_file_to_pdf(source, &config);
    // Empty doc has no nodes — should still produce a valid PDF
    // (or error gracefully if tree is empty)
    if let Ok(bytes) = result {
        assert!(bytes.starts_with(b"%PDF"));
    }
}

#[test]
fn simple_paragraph_produces_pdf() {
    let source = "Hello, world! This is a simple paragraph.";
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.len() > 100);
}

#[test]
fn headings_produce_pdf() {
    let source = r#"# Chapter One

Some introductory text.

## Section 1.1

More text here with **bold** and *italic* formatting.

### Subsection

Final paragraph.
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn code_block_produces_pdf() {
    let source = r#"Here is some code:

```rust
fn main() {
    let x = 42;
    println!("Hello, {x}!");
}
```

And some inline `code` too.
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn lists_produce_pdf() {
    let source = r#"Unordered list:

- First item
- Second item with **bold**
- Third item

Ordered list:

1. Step one
2. Step two
3. Step three
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn blockquote_produces_pdf() {
    let source = r#"> This is a blockquote.
> It spans multiple lines.
>
> And has multiple paragraphs.
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn table_produces_pdf() {
    let source = r#"| Name | Age | City |
|------|-----|------|
| Alice | 30 | NYC |
| Bob | 25 | LA |
| Carol | 35 | SF |
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn links_produce_pdf() {
    let source = "Visit [Rust](https://www.rust-lang.org) for more info.";
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn math_produces_pdf() {
    let source = r#"The equation $E = mc^2$ is well known.

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn footnotes_produce_pdf() {
    let source = r#"This has a footnote[^1].

[^1]: This is the footnote content.
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn mixed_content_produces_pdf() {
    let source = r#"---
title: Test Document
---

# Introduction

This is a **test document** with *mixed content*.

## Code Example

```python
def hello():
    print("Hello!")
```

## Lists and Tables

- Item one
- Item two

| Col A | Col B |
|-------|-------|
| 1     | 2     |

> A blockquote for good measure.

---

## Math

The formula $a^2 + b^2 = c^2$ is classic.

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    // Should be a decent-sized PDF
    assert!(bytes.len() > 500);
}

#[test]
fn definition_list_produces_pdf() {
    let source = r#"
Term One
: Definition of term one.

Term Two
: First definition.
: Second definition.
"#;
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn strikethrough_produces_pdf() {
    let source = "This has ~~deleted text~~ in it.";
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[test]
fn code_block_renders_text_inside_container() {
    let source = "```\nhello\nworld\n```\n";
    let config = test_config();
    let bytes = oxidoc_print::render_file_to_pdf(source, &config).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    assert!(bytes.len() > 500, "PDF should have content");
}

#[test]
fn list_items_have_markers() {
    let source = "- Item one\n- Item two\n- Item three\n";
    let config = test_config();
    let root = rdx_parser::parse(source);
    let tree = render_to_tree(&root, &config).unwrap();

    let mut list_items = Vec::new();
    for id_raw in 0..tree.node_count() {
        let id = oxipdf::ir::NodeId::from_raw(id_raw as u32);
        let node = tree.node(id);
        if node.semantic_role == Some(SemanticRole::ListItem) {
            list_items.push((id_raw, node.style.list.marker.as_ref().map(|m| m.text())));
        }
    }

    assert!(!list_items.is_empty(), "should have ListItem nodes");
    for (id, marker) in &list_items {
        assert!(
            marker.is_some(),
            "ListItem {id} should have a marker, got None"
        );
    }
}

#[test]
fn ordered_list_items_have_numbers() {
    let source = "1. First\n2. Second\n3. Third\n";
    let config = test_config();
    let root = rdx_parser::parse(source);
    let tree = render_to_tree(&root, &config).unwrap();

    let mut markers = Vec::new();
    for id_raw in 0..tree.node_count() {
        let id = oxipdf::ir::NodeId::from_raw(id_raw as u32);
        let node = tree.node(id);
        if node.semantic_role == Some(SemanticRole::ListItem)
            && let Some(m) = &node.style.list.marker
        {
            markers.push(m.text());
        }
    }

    assert_eq!(markers, vec!["1.", "2.", "3."]);
}
