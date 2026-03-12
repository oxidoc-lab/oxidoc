use crate::crawler::NavGroup;
use crate::error::Result;

use super::types::PageContent;

/// Extract searchable plain text from all pages in navigation groups.
pub fn extract_page_text(nav_groups: &[NavGroup]) -> Result<Vec<PageContent>> {
    let mut pages = Vec::new();

    for nav_group in nav_groups {
        for page_entry in &nav_group.pages {
            let content = std::fs::read_to_string(&page_entry.file_path).map_err(|e| {
                crate::error::OxidocError::FileRead {
                    path: page_entry.file_path.display().to_string(),
                    source: e,
                }
            })?;

            let root = rdx_parser::parse(&content);
            let title = extract_page_title(&root).unwrap_or_else(|| page_entry.title.clone());
            let text = extract_searchable_text(&root);

            pages.push(PageContent {
                title,
                slug: page_entry.slug.clone(),
                text,
            });
        }
    }

    Ok(pages)
}

/// Extract the page title from the first h1 heading.
fn extract_page_title(root: &rdx_ast::Root) -> Option<String> {
    for node in &root.children {
        if let rdx_ast::Node::Heading(h) = node
            && h.depth.unwrap_or(1) == 1
        {
            return Some(crate::utils::extract_plain_text(node));
        }
    }
    None
}

/// Extract all searchable plain text from a document.
/// Includes headings, paragraphs, list items, and inline code.
fn extract_searchable_text(root: &rdx_ast::Root) -> String {
    let mut text = String::new();

    for node in &root.children {
        append_searchable_content(&mut text, node);
    }

    text
}

/// Recursively append searchable content from a node, filtering out code blocks.
fn append_searchable_content(text: &mut String, node: &rdx_ast::Node) {
    use rdx_ast::Node;

    match node {
        Node::Heading(_) | Node::Paragraph(_) | Node::ListItem(_) => {
            text.push_str(&crate::utils::extract_plain_text(node));
            text.push(' ');
        }
        Node::CodeBlock(_) => {
            // Skip code blocks for plain text extraction
        }
        Node::Blockquote(_) | Node::Table(_) => {
            text.push_str(&crate::utils::extract_plain_text(node));
            text.push(' ');
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    append_searchable_content(text, child);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_page_title() {
        let root = rdx_parser::parse("# Main Title\n\nSome content here");
        let title = extract_page_title(&root);
        assert_eq!(title, Some("Main Title".to_string()));
    }

    #[test]
    fn test_extract_page_title_no_h1() {
        let root = rdx_parser::parse("## Subtitle\n\nNo main title");
        let title = extract_page_title(&root);
        assert_eq!(title, None);
    }

    #[test]
    fn test_extract_searchable_text_basic() {
        let root = rdx_parser::parse("# Heading\n\nParagraph text");
        let text = extract_searchable_text(&root);
        assert!(text.contains("Heading"));
        assert!(text.contains("Paragraph"));
        assert!(text.contains("text"));
    }

    #[test]
    fn test_extract_searchable_text_with_formatting() {
        let root = rdx_parser::parse("# Title\n\nThis is **bold** and `code` text");
        let text = extract_searchable_text(&root);
        assert!(text.contains("Title"));
        assert!(text.contains("bold"));
        assert!(text.contains("code"));
    }

    #[test]
    fn test_extract_searchable_text_empty_document() {
        let root = rdx_parser::parse("");
        let text = extract_searchable_text(&root);
        assert!(text.is_empty() || text.chars().all(|c| c.is_whitespace()));
    }

    #[test]
    fn test_extract_page_text_integration() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            docs.join("test.rdx"),
            "# Test Page\n\nThis is test content with **bold** text.",
        )
        .unwrap();

        let nav_groups = vec![crate::crawler::NavGroup {
            title: "Test".to_string(),
            pages: vec![crate::crawler::PageEntry {
                title: "Test".to_string(),
                slug: "test".to_string(),
                file_path: docs.join("test.rdx"),
                group: None,
            }],
        }];

        let pages = extract_page_text(&nav_groups).unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].slug, "test");
        assert_eq!(pages[0].title, "Test Page");
        assert!(pages[0].text.contains("Test"));
        assert!(pages[0].text.contains("content"));
    }
}
