use crate::crawler::PageEntry;
use crate::error::{OxidocError, Result};
use crate::template_parts::{PageGitMeta, PageNav};

/// Check for error nodes in the parsed AST.
pub(crate) fn check_parse_errors(root: &rdx_ast::Root, path: &str) -> Result<()> {
    for node in &root.children {
        if let rdx_ast::Node::Error(e) = node {
            return Err(OxidocError::RdxParse {
                path: path.into(),
                message: e.message.clone(),
            });
        }
    }
    Ok(())
}

/// Extract the page title by reading and parsing an RDX file.
pub(crate) fn extract_page_title_from_file(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let root = rdx_parser::parse(&content);
    extract_page_title(&root)
}

/// Extract the page title from the first h1 heading.
pub(crate) fn extract_page_title(root: &rdx_ast::Root) -> Option<String> {
    for node in &root.children {
        if let rdx_ast::Node::Heading(h) = node
            && h.depth.unwrap_or(1) == 1
        {
            return Some(crate::toc::extract_heading_text(node));
        }
    }
    None
}

/// Build prev/next navigation links for a page given its position in the flat page list.
pub(crate) fn build_page_nav(
    slug: &str,
    slug_index: &std::collections::HashMap<String, usize>,
    pages: &[PageEntry],
) -> PageNav {
    if let Some(&idx) = slug_index.get(slug) {
        PageNav {
            prev: if idx > 0 {
                let p = &pages[idx - 1];
                Some((
                    p.slug.clone(),
                    extract_page_title_from_file(&p.file_path).unwrap_or_else(|| p.title.clone()),
                ))
            } else {
                None
            },
            next: if idx + 1 < pages.len() {
                let p = &pages[idx + 1];
                Some((
                    p.slug.clone(),
                    extract_page_title_from_file(&p.file_path).unwrap_or_else(|| p.title.clone()),
                ))
            } else {
                None
            },
        }
    } else {
        PageNav::default()
    }
}

/// Resolve git metadata for a page file, with filesystem mtime fallback.
pub(crate) fn resolve_git_meta(file_path: &std::path::Path) -> PageGitMeta {
    crate::utils::git_file_meta(file_path)
        .map(|(date, author)| PageGitMeta {
            last_updated: Some(date),
            last_author: Some(author),
        })
        .unwrap_or_else(|| {
            let date = std::fs::metadata(file_path)
                .and_then(|m| m.modified())
                .ok()
                .map(|t| {
                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                    dt.format("%b %-d, %Y").to_string()
                });
            PageGitMeta {
                last_updated: date,
                last_author: None,
            }
        })
}

/// Extract a description from the first paragraph of content.
pub(crate) fn extract_page_description(root: &rdx_ast::Root) -> Option<String> {
    for node in &root.children {
        if let rdx_ast::Node::Paragraph(_) = node {
            let text = crate::utils::extract_plain_text(node);
            if !text.trim().is_empty() {
                // Limit to 160 characters for SEO meta description
                return Some(text.chars().take(160).collect());
            }
        }
    }
    None
}
