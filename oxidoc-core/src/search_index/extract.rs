use crate::crawler::NavGroup;
use crate::error::Result;
use crate::page_extract::extract_page_title;
use crate::utils::heading_anchor;

use super::types::{HeadingPos, PageContent};

/// Extract searchable plain text from all pages in navigation groups.
/// Produces one entry per page with heading positions for section resolution at search time.
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

            // Extract full text and heading positions simultaneously
            let (text, headings) = extract_text_with_headings(&root);

            pages.push(PageContent {
                title,
                slug: page_entry.slug.clone(),
                text,
                headings,
            });
        }
    }

    Ok(pages)
}

/// Extract all searchable text and record heading positions (offset into the text).
fn extract_text_with_headings(root: &rdx_ast::Root) -> (String, Vec<HeadingPos>) {
    let mut text = String::new();
    let mut headings = Vec::new();

    for node in &root.children {
        if let rdx_ast::Node::Heading(h) = node {
            let depth = h.depth.unwrap_or(1);
            if depth >= 2 {
                let heading_title = crate::utils::extract_plain_text(node);
                let anchor = heading_anchor(&heading_title);
                headings.push(HeadingPos {
                    title: heading_title,
                    anchor,
                    depth,
                    offset: text.len(),
                });
            }
        }
        append_searchable_content(&mut text, node);
    }

    (text, headings)
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
    fn test_extract_text_with_headings() {
        let root = rdx_parser::parse(
            "# Title\n\nIntro text\n\n## Setup\n\nSetup content\n\n## Usage\n\nUsage content",
        );
        let (text, headings) = extract_text_with_headings(&root);
        assert!(text.contains("Intro text"));
        assert!(text.contains("Setup content"));
        assert!(text.contains("Usage content"));
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].title, "Setup");
        assert_eq!(headings[0].depth, 2);
        assert_eq!(headings[1].title, "Usage");
        // Offsets should be increasing
        assert!(headings[1].offset > headings[0].offset);
    }

    #[test]
    fn test_heading_anchor() {
        assert_eq!(heading_anchor("Getting Started"), "getting-started");
        assert_eq!(heading_anchor("Hello, World!"), "hello-world");
    }

    #[test]
    fn test_extract_page_text_integration() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            docs.join("test.rdx"),
            "# Test Page\n\nThis is test content with **bold** text.\n\n## Section One\n\nSection content here.",
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
        // One entry per page
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "Test Page");
        assert!(pages[0].text.contains("test content"));
        assert!(pages[0].text.contains("Section content"));
        assert_eq!(pages[0].headings.len(), 1);
        assert_eq!(pages[0].headings[0].title, "Section One");
    }
}
