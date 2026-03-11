use crate::config::load_config;
use crate::crawler::{NavGroup, discover_pages};
use crate::error::{OxidocError, Result};
use crate::renderer::render_document;
use crate::template::{render_page, render_sidebar};
use crate::toc::{extract_toc, render_toc};
use std::path::Path;

/// Result of a successful site build.
#[derive(Debug)]
pub struct BuildResult {
    /// Total number of pages rendered in this build.
    pub pages_rendered: usize,
    /// Absolute path to the output directory.
    pub output_dir: String,
}

/// Build the documentation site from a project root to an output directory.
pub fn build_site(project_root: &Path, output_dir: &Path) -> Result<BuildResult> {
    let config = load_config(project_root)?;
    let nav_groups = discover_pages(project_root, &config)?;

    std::fs::create_dir_all(output_dir).map_err(|e| OxidocError::DirCreate {
        path: output_dir.display().to_string(),
        source: e,
    })?;

    let mut pages_rendered = 0;

    for group in &nav_groups {
        for page in &group.pages {
            let content =
                std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                    path: page.file_path.display().to_string(),
                    source: e,
                })?;

            let root = rdx_parser::parse(&content);
            check_parse_errors(&root, &page.file_path.display().to_string())?;

            let content_html = render_document(&root, &config.components.custom);
            let toc_entries = extract_toc(&root);
            let toc_html = render_toc(&toc_entries);
            let sidebar_with_active = render_sidebar(&nav_groups, &page.slug);

            let page_title = extract_page_title(&root).unwrap_or_else(|| page.title.clone());
            let full_html = render_page(
                &config,
                &page_title,
                &content_html,
                &toc_html,
                &sidebar_with_active,
                &page.slug,
            );

            let page_output = output_dir.join(format!("{}.html", page.slug));
            if let Some(parent) = page_output.parent() {
                std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }

            std::fs::write(&page_output, full_html).map_err(|e| OxidocError::FileWrite {
                path: page_output.display().to_string(),
                source: e,
            })?;

            pages_rendered += 1;
            tracing::info!(page = %page.slug, "Rendered");
        }
    }

    generate_llms_txt(&nav_groups, output_dir)?;
    generate_index_redirect(&nav_groups, output_dir)?;

    Ok(BuildResult {
        pages_rendered,
        output_dir: output_dir.display().to_string(),
    })
}

/// Check for error nodes in the parsed AST.
fn check_parse_errors(root: &rdx_ast::Root, path: &str) -> Result<()> {
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

/// Extract the page title from the first h1 heading.
fn extract_page_title(root: &rdx_ast::Root) -> Option<String> {
    for node in &root.children {
        if let rdx_ast::Node::Heading(h) = node
            && h.depth.unwrap_or(1) == 1
        {
            return Some(crate::toc::extract_heading_text(node));
        }
    }
    None
}

/// Generate `llms.txt` and `llms-full.txt` for AI/RAG consumption.
fn generate_llms_txt(nav_groups: &[NavGroup], output_dir: &Path) -> Result<()> {
    let mut summary = String::new();
    let mut full = String::new();

    for group in nav_groups {
        for page in &group.pages {
            let content =
                std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                    path: page.file_path.display().to_string(),
                    source: e,
                })?;

            summary.push_str(&format!("- /{}: {}\n", page.slug, page.title));
            full.push_str(&format!(
                "\n---\n# {} ({})\n\n{}\n",
                page.title, page.slug, content
            ));
        }
    }

    std::fs::write(output_dir.join("llms.txt"), summary).map_err(|e| OxidocError::FileWrite {
        path: output_dir.join("llms.txt").display().to_string(),
        source: e,
    })?;

    std::fs::write(output_dir.join("llms-full.txt"), full).map_err(|e| OxidocError::FileWrite {
        path: output_dir.join("llms-full.txt").display().to_string(),
        source: e,
    })?;

    Ok(())
}

/// Generate an index.html that redirects to the first page.
fn generate_index_redirect(nav_groups: &[NavGroup], output_dir: &Path) -> Result<()> {
    let first_slug = nav_groups
        .iter()
        .flat_map(|g| g.pages.first())
        .map(|p| p.slug.as_str())
        .next();

    if let Some(slug) = first_slug {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=/{slug}.html"></head>
<body><a href="/{slug}.html">Redirecting...</a></body>
</html>"#
        );
        std::fs::write(output_dir.join("index.html"), html).map_err(|e| {
            OxidocError::FileWrite {
                path: output_dir.join("index.html").display().to_string(),
                source: e,
            }
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_site_end_to_end() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            root.join("oxidoc.toml"),
            r#"[project]
name = "Test Project"
"#,
        )
        .unwrap();
        std::fs::write(
            docs.join("intro.rdx"),
            "# Introduction\n\nWelcome to the docs.\n\n## Getting Started\n\nHere we go.",
        )
        .unwrap();
        std::fs::write(docs.join("setup.rdx"), "# Setup\n\nInstall the tool.\n").unwrap();

        let output = root.join("dist");
        let result = build_site(root, &output).unwrap();

        assert_eq!(result.pages_rendered, 2);
        assert!(output.join("intro.html").exists());
        assert!(output.join("setup.html").exists());
        assert!(output.join("index.html").exists());
        assert!(output.join("llms.txt").exists());
        assert!(output.join("llms-full.txt").exists());

        let intro_html = std::fs::read_to_string(output.join("intro.html")).unwrap();
        assert!(intro_html.contains("Introduction"));
        assert!(intro_html.contains("<!DOCTYPE html>"));
        assert!(intro_html.contains("Test Project"));
        assert!(intro_html.contains(r#"id="getting-started""#));
        assert!(intro_html.contains("oxidoc-toc"));

        let llms = std::fs::read_to_string(output.join("llms.txt")).unwrap();
        assert!(llms.contains("/intro"));
        assert!(llms.contains("/setup"));
    }

    #[test]
    fn build_site_missing_config() {
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path().join("dist");
        let err = build_site(tmp.path(), &output).unwrap_err();
        assert!(matches!(err, OxidocError::ConfigRead { .. }));
    }

    #[test]
    fn build_site_explicit_routing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            root.join("oxidoc.toml"),
            r#"
[project]
name = "Routed Docs"

[routing]
navigation = [
  { group = "Guide", pages = ["quickstart"] }
]
"#,
        )
        .unwrap();
        std::fs::write(docs.join("quickstart.rdx"), "# Quickstart\n\nGo!").unwrap();

        let output = root.join("dist");
        let result = build_site(root, &output).unwrap();
        assert_eq!(result.pages_rendered, 1);
        assert!(output.join("quickstart.html").exists());

        let html = std::fs::read_to_string(output.join("quickstart.html")).unwrap();
        assert!(html.contains("Guide"));
    }

    #[test]
    fn build_site_missing_page_in_routing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            root.join("oxidoc.toml"),
            r#"
[project]
name = "Bad Routing"

[routing]
navigation = [
  { group = "Guide", pages = ["nonexistent"] }
]
"#,
        )
        .unwrap();

        let output = root.join("dist");
        let err = build_site(root, &output).unwrap_err();
        assert!(matches!(err, OxidocError::PageNotFound { .. }));
    }
}
