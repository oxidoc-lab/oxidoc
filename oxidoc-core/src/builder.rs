use crate::asset_hash::hash_content;
use crate::assets::copy_assets;
use crate::breadcrumb::{generate_breadcrumbs, render_breadcrumbs};
use crate::config::load_config;
use crate::crawler::discover_pages;
use crate::css::{generate_base_css, minify_css};
use crate::error::{OxidocError, Result};
use crate::feed::generate_feed;
use crate::incremental::IncrementalCache;
use crate::loader::generate_loader_js;
use crate::minify::minify_html;
use crate::openapi;
use crate::outputs::{generate_index_redirect, generate_llms_txt, generate_redirects};
use crate::renderer::render_document;
use crate::sitemap::{generate_robots_txt, generate_sitemap};
use crate::sri::generate_sri_hash;
use crate::template::{AssetConfig, render_404_page, render_page};
use crate::template_parts::render_sidebar;
use crate::toc::{extract_toc, render_toc};
use crate::versioning::VersioningState;
use rayon::prelude::*;
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

    // Load incremental cache
    let mut cache = IncrementalCache::load(output_dir)?;

    // Setup versioning (reserved for future use when implementing per-version builds)
    let _versioning = VersioningState::from_config(&config.versioning);

    // Generate assets (CSS and JS)
    let css = generate_base_css(&config);
    let css = minify_css(&css);
    let js = generate_loader_js();

    // Hash assets for cache busting
    let css_hash = hash_content(css.as_bytes());
    let js_hash = hash_content(js.as_bytes());
    let css_filename = format!("oxidoc.{}.css", css_hash);
    let js_filename = format!("oxidoc-loader.{}.js", js_hash);
    let css_path = format!("/{}", css_filename);
    let js_path = format!("/{}", js_filename);

    // Generate SRI hashes for security
    let css_sri = generate_sri_hash(css.as_bytes());
    let js_sri = generate_sri_hash(js.as_bytes());

    let assets = AssetConfig {
        css_path: Some(&css_path),
        js_path: Some(&js_path),
        css_sri: Some(&css_sri),
        js_sri: Some(&js_sri),
    };

    // Write hashed assets
    std::fs::write(output_dir.join(&css_filename), css.as_bytes()).map_err(|e| {
        OxidocError::FileWrite {
            path: output_dir.join(&css_filename).display().to_string(),
            source: e,
        }
    })?;

    std::fs::write(output_dir.join(&js_filename), js.as_bytes()).map_err(|e| {
        OxidocError::FileWrite {
            path: output_dir.join(&js_filename).display().to_string(),
            source: e,
        }
    })?;

    // Build pages using rayon parallelization
    let pages_to_build: Vec<_> = nav_groups.iter().flat_map(|g| g.pages.iter()).collect();

    let results: Result<Vec<_>> = pages_to_build
        .par_iter()
        .map(|page| {
            let content =
                std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                    path: page.file_path.display().to_string(),
                    source: e,
                })?;

            // Check if rebuild is needed
            if !cache.needs_rebuild(&page.file_path.display().to_string(), content.as_bytes()) {
                return Ok(false);
            }

            let root = rdx_parser::parse(&content);
            check_parse_errors(&root, &page.file_path.display().to_string())?;

            let content_html = render_document(&root, &config.components.custom);
            let toc_entries = extract_toc(&root);
            let toc_html = render_toc(&toc_entries);
            let sidebar_with_active = render_sidebar(&nav_groups, &page.slug);
            let breadcrumbs = generate_breadcrumbs(&page.slug);
            let breadcrumb_html = render_breadcrumbs(&breadcrumbs);

            let page_title = extract_page_title(&root).unwrap_or_else(|| page.title.clone());
            let page_description = extract_page_description(&root);

            let full_html = render_page(
                &config,
                &page_title,
                &content_html,
                &toc_html,
                &sidebar_with_active,
                &breadcrumb_html,
                &page.slug,
                page_description.as_deref(),
                &assets,
            );

            let page_output = output_dir.join(format!("{}.html", page.slug));
            if let Some(parent) = page_output.parent() {
                std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }

            let minified_html = minify_html(&full_html);
            std::fs::write(&page_output, minified_html).map_err(|e| OxidocError::FileWrite {
                path: page_output.display().to_string(),
                source: e,
            })?;

            tracing::info!(page = %page.slug, "Rendered");
            Ok(true)
        })
        .collect();

    let rendered_flags = results?;
    let mut pages_rendered = rendered_flags.iter().filter(|&&b| b).count();

    // Update cache with rendered pages
    for page in &pages_to_build {
        let content =
            std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                path: page.file_path.display().to_string(),
                source: e,
            })?;
        cache.record(&page.file_path.display().to_string(), content.as_bytes());
    }

    // Process OpenAPI specs from navigation groups
    for nav_group_cfg in &config.routing.navigation {
        if let Some(ref spec_path) = nav_group_cfg.openapi {
            let resolved = project_root.join(spec_path);
            let spec = openapi::load_openapi_spec(&resolved)?;
            let api_nav = openapi::generate_api_nav_groups(
                &openapi::extract_endpoints(&spec),
                &nav_group_cfg.group,
            );
            let mut combined_nav = nav_groups.clone();
            combined_nav.extend(api_nav);

            let api_count =
                openapi::build_api_pages(&spec, output_dir, &config, &combined_nav, &assets)?;
            pages_rendered += api_count;
        }
    }

    let assets_copied = copy_assets(project_root, output_dir)?;
    if assets_copied > 0 {
        tracing::info!(count = assets_copied, "Assets copied");
    }

    generate_llms_txt(&nav_groups, output_dir)?;
    generate_index_redirect(&nav_groups, output_dir)?;

    // Generate SEO files
    let base_url = config.project.base_url.as_deref().unwrap_or("/");
    generate_sitemap(&nav_groups, base_url, output_dir)?;
    generate_robots_txt(base_url, output_dir)?;
    let description = config
        .project
        .description
        .as_deref()
        .unwrap_or("Documentation");
    generate_feed(
        &nav_groups,
        &config.project.name,
        base_url,
        description,
        output_dir,
    )?;

    // Generate 404 page
    let not_found_html = render_404_page(&config, &assets);
    let not_found_minified = minify_html(&not_found_html);
    std::fs::write(output_dir.join("404.html"), not_found_minified).map_err(|e| {
        OxidocError::FileWrite {
            path: output_dir.join("404.html").display().to_string(),
            source: e,
        }
    })?;

    // Generate redirect pages
    generate_redirects(&config.redirects.redirects, output_dir)?;

    // Save incremental cache
    cache.save(output_dir)?;

    // Generate search indices
    crate::search_index::generate_search_index(&nav_groups, output_dir, &config.search)?;

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

/// Extract a description from the first paragraph of content.
fn extract_page_description(root: &rdx_ast::Root) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_project(
        config_toml: &str,
        files: &[(&str, &str)],
    ) -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(root.join("oxidoc.toml"), config_toml).unwrap();
        for (name, content) in files {
            std::fs::write(docs.join(name), content).unwrap();
        }
        let output = root.join("dist");
        (tmp, output)
    }

    fn find_hashed_file(dir: &Path, prefix: &str, ext: &str) -> std::path::PathBuf {
        std::fs::read_dir(dir)
            .unwrap()
            .find_map(|e| {
                let p = e.unwrap().path();
                let name = p.file_name().unwrap().to_string_lossy().to_string();
                (name.starts_with(prefix) && name.ends_with(ext)).then_some(p)
            })
            .unwrap_or_else(|| panic!("No {prefix}*{ext} found"))
    }

    #[test]
    fn build_site_end_to_end() {
        let (tmp, output) = setup_project(
            "[project]\nname = \"Test Project\"\n",
            &[
                (
                    "intro.rdx",
                    "# Introduction\n\nWelcome to the docs.\n\n## Getting Started\n\nHere we go.",
                ),
                ("setup.rdx", "# Setup\n\nInstall the tool.\n"),
            ],
        );
        let result = build_site(tmp.path(), &output).unwrap();

        assert_eq!(result.pages_rendered, 2);
        for f in [
            "intro.html",
            "setup.html",
            "index.html",
            "llms.txt",
            "llms-full.txt",
            "sitemap.xml",
            "robots.txt",
            "404.html",
            ".oxidoc-cache.json",
            "search-lexical.json",
        ] {
            assert!(output.join(f).exists(), "{f} should exist");
        }

        let css = std::fs::read_to_string(find_hashed_file(&output, "oxidoc.", ".css")).unwrap();
        assert!(css.contains("oxidoc-primary"));

        let js =
            std::fs::read_to_string(find_hashed_file(&output, "oxidoc-loader.", ".js")).unwrap();
        assert!(js.contains("oxidoc-registry.wasm"));

        let intro_html = std::fs::read_to_string(output.join("intro.html")).unwrap();
        assert!(intro_html.contains("Introduction"));
        assert!(intro_html.contains("<!DOCTYPE html>"));
        assert!(intro_html.contains("Test Project"));

        let llms = std::fs::read_to_string(output.join("llms.txt")).unwrap();
        assert!(llms.contains("/intro") && llms.contains("/setup"));

        let sitemap = std::fs::read_to_string(output.join("sitemap.xml")).unwrap();
        assert!(sitemap.contains("intro.html") && sitemap.contains("setup.html"));

        let robots = std::fs::read_to_string(output.join("robots.txt")).unwrap();
        assert!(robots.contains("User-agent: *") && robots.contains("Sitemap:"));

        let not_found = std::fs::read_to_string(output.join("404.html")).unwrap();
        assert!(not_found.contains("404") && not_found.contains("Not Found"));
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
        let routing_toml = "[project]\nname = \"Routed Docs\"\n\n[routing]\nnavigation = [\n  { group = \"Guide\", pages = [\"quickstart\"] }\n]\n";
        let (tmp, output) =
            setup_project(routing_toml, &[("quickstart.rdx", "# Quickstart\n\nGo!")]);
        let result = build_site(tmp.path(), &output).unwrap();
        assert_eq!(result.pages_rendered, 1);
        assert!(output.join("quickstart.html").exists());
        let html = std::fs::read_to_string(output.join("quickstart.html")).unwrap();
        assert!(html.contains("Guide"));
    }

    #[test]
    fn build_site_missing_page_in_routing() {
        let routing_toml = "[project]\nname = \"Bad Routing\"\n\n[routing]\nnavigation = [\n  { group = \"Guide\", pages = [\"nonexistent\"] }\n]\n";
        let (tmp, output) = setup_project(routing_toml, &[]);
        let err = build_site(tmp.path(), &output).unwrap_err();
        assert!(matches!(err, OxidocError::PageNotFound { .. }));
    }

    #[test]
    fn llms_txt_generation() {
        let (tmp, output) = setup_project(
            "[project]\nname = \"Test\"\n",
            &[(
                "guide.rdx",
                "# User Guide\n\nThis is a comprehensive guide.",
            )],
        );
        let _ = build_site(tmp.path(), &output).unwrap();

        let llms = std::fs::read_to_string(output.join("llms.txt")).unwrap();
        assert!(llms.contains("- /guide:"));

        let full = std::fs::read_to_string(output.join("llms-full.txt")).unwrap();
        assert!(full.contains("User Guide"));
        assert!(full.contains("(guide)"));
        assert!(full.contains("comprehensive guide"));
    }
}
