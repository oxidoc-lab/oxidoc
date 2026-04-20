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

pub(crate) fn extract_frontmatter_title(root: &rdx_ast::Root) -> Option<String> {
    root.frontmatter
        .as_ref()?
        .get("title")?
        .as_str()
        .map(|s| s.to_string())
}

pub(crate) fn extract_frontmatter_short_title(root: &rdx_ast::Root) -> Option<String> {
    root.frontmatter
        .as_ref()?
        .get("short_title")?
        .as_str()
        .map(|s| s.to_string())
}

pub(crate) fn extract_frontmatter_description(root: &rdx_ast::Root) -> Option<String> {
    root.frontmatter
        .as_ref()?
        .get("description")?
        .as_str()
        .map(|s| s.to_string())
}

/// Extract the page title, preferring frontmatter over the first h1 heading.
pub(crate) fn extract_page_title(root: &rdx_ast::Root) -> Option<String> {
    if let Some(t) = extract_frontmatter_title(root) {
        return Some(t);
    }
    for node in &root.children {
        if let rdx_ast::Node::Heading(h) = node
            && h.depth.unwrap_or(1) == 1
        {
            return Some(crate::toc::extract_heading_text(node));
        }
    }
    None
}

/// Pre-compute a map of slug → page title for all pages.
pub(crate) fn build_title_map(pages: &[PageEntry]) -> std::collections::HashMap<String, String> {
    pages
        .iter()
        .map(|p| {
            let title =
                extract_page_title_from_file(&p.file_path).unwrap_or_else(|| p.title.clone());
            (p.slug.clone(), title)
        })
        .collect()
}

/// Build prev/next navigation links for a page given its position in the flat page list.
pub(crate) fn build_page_nav(
    slug: &str,
    slug_index: &std::collections::HashMap<String, usize>,
    pages: &[PageEntry],
    title_map: &std::collections::HashMap<String, String>,
) -> PageNav {
    if let Some(&idx) = slug_index.get(slug) {
        PageNav {
            prev: if idx > 0 {
                let p = &pages[idx - 1];
                Some((
                    p.slug.clone(),
                    title_map
                        .get(&p.slug)
                        .cloned()
                        .unwrap_or_else(|| p.title.clone()),
                ))
            } else {
                None
            },
            next: if idx + 1 < pages.len() {
                let p = &pages[idx + 1];
                Some((
                    p.slug.clone(),
                    title_map
                        .get(&p.slug)
                        .cloned()
                        .unwrap_or_else(|| p.title.clone()),
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

/// Extract the page layout from frontmatter (e.g. "landing").
pub(crate) fn extract_page_layout(root: &rdx_ast::Root) -> Option<String> {
    root.frontmatter
        .as_ref()?
        .get("layout")?
        .as_str()
        .map(|s| s.to_string())
}

/// Extract a description, preferring frontmatter over the first paragraph of content.
pub(crate) fn extract_page_description(root: &rdx_ast::Root) -> Option<String> {
    if let Some(d) = extract_frontmatter_description(root) {
        return Some(d);
    }
    for node in &root.children {
        if let rdx_ast::Node::Paragraph(_) = node {
            let text = crate::utils::extract_plain_text(node);
            if !text.trim().is_empty() {
                return Some(text.chars().take(160).collect());
            }
        }
    }
    None
}
