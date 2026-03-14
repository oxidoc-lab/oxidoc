use crate::asset_hash::hash_content;
use crate::assets::copy_assets;
use crate::breadcrumb::{generate_breadcrumbs, render_breadcrumbs};
use crate::config::load_config;
use crate::crawler::{discover_pages, discover_sections};
use crate::css::{generate_base_css, minify_css};
use crate::error::{OxidocError, Result};
use crate::i18n::{I18nState, load_translations};
use crate::incremental::IncrementalCache;
use crate::loader::generate_loader_js;
use crate::minify::minify_html;
use crate::openapi;
use crate::outputs::{
    generate_index_redirect, generate_llms_txt, generate_redirects, generate_seo_files,
};
use crate::page_extract::{
    build_page_nav, check_parse_errors, extract_page_description, extract_page_layout,
    extract_page_title, resolve_git_meta,
};
use crate::renderer::render_document;
use crate::search_provider::SearchProvider;
use crate::sri::generate_sri_hash;
use crate::template::render_page;
use crate::template_404::render_404_page;
use crate::template_assets::AssetConfig;
use crate::template_landing::render_landing_page;
use crate::template_parts::{render_page_meta, render_sidebar_with_homepage};
use crate::theme;
use crate::toc::{extract_toc, render_toc};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub use crate::build_result::BuildResult;

/// Build the documentation site from a project root to an output directory.
pub fn build_site(project_root: &Path, output_dir: &Path) -> Result<BuildResult> {
    build_site_with_model(project_root, output_dir, None)
}

/// Build with an optional bundled GGUF embedding model for semantic search.
pub fn build_site_with_model(
    project_root: &Path,
    output_dir: &Path,
    bundled_model: Option<&[u8]>,
) -> Result<BuildResult> {
    let config = load_config(project_root)?;
    let mut sections = discover_sections(project_root, &config)?;
    let mut nav_groups = discover_pages(project_root, &config)?;

    std::fs::create_dir_all(output_dir).map_err(|e| OxidocError::DirCreate {
        path: output_dir.display().to_string(),
        source: e,
    })?;

    // Load incremental cache
    let mut cache = IncrementalCache::load(output_dir)?;

    // Resolve search provider from config
    let search_provider = SearchProvider::from_config(&config.search)?;

    // Setup i18n
    let i18n_state = I18nState::from_config(&config.i18n.default_locale, &config.i18n.locales);
    let translation_bundles = if i18n_state.enabled {
        load_translations(
            project_root,
            &config.i18n.translation_dir,
            &i18n_state.locales,
        )?
    } else {
        HashMap::new()
    };

    // Resolve theme
    let resolved_theme = theme::resolve_theme(
        config.theme.primary.as_deref(),
        config.theme.accent.as_deref(),
        config.theme.font.as_deref(),
        config.theme.code_font.as_deref(),
    );

    // Load custom CSS files if configured
    let custom_css = if config.theme.custom_css.is_empty() {
        None
    } else {
        let mut combined = String::new();
        for css_path in &config.theme.custom_css {
            let custom_path = if Path::new(css_path).is_absolute() {
                Path::new(css_path).to_path_buf()
            } else {
                project_root.join(css_path)
            };
            let content =
                std::fs::read_to_string(&custom_path).map_err(|e| OxidocError::FileRead {
                    path: custom_path.display().to_string(),
                    source: e,
                })?;
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&content);
        }
        Some(combined)
    };

    // Generate assets (CSS and JS)
    let css = generate_base_css(
        &resolved_theme,
        &config.theme.dark_mode,
        custom_css.as_deref(),
    );
    let css = minify_css(&css);
    // Hash assets for cache busting
    let css_hash = hash_content(css.as_bytes());
    let css_filename = format!("oxidoc.{}.css", css_hash);
    let css_path = format!("/{}", css_filename);
    let css_sri = generate_sri_hash(css.as_bytes());

    let js = generate_loader_js();
    let js_hash = hash_content(js.as_bytes());
    let js_filename = format!("oxidoc-loader.{}.js", js_hash);
    let js_path = format!("/{}", js_filename);
    let js_sri = generate_sri_hash(js.as_bytes());

    std::fs::write(output_dir.join(&js_filename), js.as_bytes()).map_err(|e| {
        OxidocError::FileWrite {
            path: output_dir.join(&js_filename).display().to_string(),
            source: e,
        }
    })?;

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

    // Patch nav group page titles from frontmatter so the sidebar uses them
    for group in &mut nav_groups {
        for page in &mut group.pages {
            if let Ok(content) = std::fs::read_to_string(&page.file_path) {
                let root = rdx_parser::parse(&content);
                if let Some(title) = extract_page_title(&root) {
                    page.title = title;
                }
            }
        }
    }

    // Patch section nav group page titles from frontmatter
    for section in &mut sections {
        for group in &mut section.nav_groups {
            for page in &mut group.pages {
                if let Ok(content) = std::fs::read_to_string(&page.file_path) {
                    let root = rdx_parser::parse(&content);
                    if let Some(title) = extract_page_title(&root) {
                        page.title = title;
                    }
                }
            }
        }
    }

    // Build a map from page slug to the section's nav groups for section-specific sidebars
    let section_nav_map: HashMap<String, Vec<crate::crawler::NavGroup>> = {
        let mut map = HashMap::new();
        for section in &sections {
            for page in section.nav_groups.iter().flat_map(|g| &g.pages) {
                map.insert(page.slug.clone(), section.nav_groups.clone());
            }
        }
        map
    };
    let section_nav_map = Arc::new(section_nav_map);

    // Build pages for each locale
    let build_locales = i18n_state.build_locales();
    let pages_to_build: Vec<_> = nav_groups.iter().flat_map(|g| g.pages.iter()).collect();

    // Root pages config — standalone pages at `/`
    let root_config = config.routing.root.clone();
    let has_root = root_config.is_some();
    // Legacy compat: homepage_slug is None when using root config
    let homepage_slug: Option<String> = None;

    let config = Arc::new(config);
    let i18n_state = Arc::new(i18n_state);
    let search_provider = Arc::new(search_provider);
    let mut pages_rendered_total = 0;

    // Read and parse all pages once, check cache, then render across locales
    let page_contents: Vec<_> = pages_to_build
        .iter()
        .filter_map(|page| {
            let content = std::fs::read_to_string(&page.file_path).ok()?;
            let needs_rebuild =
                cache.needs_rebuild(&page.file_path.display().to_string(), content.as_bytes());
            if needs_rebuild {
                Some((page, content))
            } else {
                None
            }
        })
        .collect();

    // Build flat page list and slug-to-index map for prev/next navigation
    let flat_pages: Vec<crate::crawler::PageEntry> =
        pages_to_build.iter().map(|p| (*p).clone()).collect();
    let slug_index: HashMap<String, usize> = flat_pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.slug.clone(), i))
        .collect();
    let slug_index = Arc::new(slug_index);
    let flat_pages = Arc::new(flat_pages);

    let nav_groups_arc = Arc::new(nav_groups.clone());

    for locale in &build_locales {
        let locale_output_dir = if i18n_state.is_default_locale(locale) {
            output_dir.to_path_buf()
        } else {
            output_dir.join(locale)
        };

        std::fs::create_dir_all(&locale_output_dir).map_err(|e| OxidocError::DirCreate {
            path: locale_output_dir.display().to_string(),
            source: e,
        })?;

        let config_arc = Arc::clone(&config);
        let i18n_state_arc = Arc::clone(&i18n_state);
        let search_provider_arc = Arc::clone(&search_provider);
        let nav_groups_arc = Arc::clone(&nav_groups_arc);
        let section_nav_map_arc = Arc::clone(&section_nav_map);
        let slug_index_arc = Arc::clone(&slug_index);
        let pages_arc = Arc::clone(&flat_pages);
        let locale_str = locale.clone();

        let results: Result<Vec<_>> = page_contents
            .par_iter()
            .map(|(page, content)| {
                let root = rdx_parser::parse(content);
                check_parse_errors(&root, &page.file_path.display().to_string())?;

                let content_html = render_document(
                    &root,
                    &config_arc.components.custom,
                    config_arc.project.debug_islands,
                );
                let toc_entries = extract_toc(&root);
                let toc_html = render_toc(&toc_entries);
                let sidebar_groups = section_nav_map_arc
                    .get(&page.slug)
                    .map(|g| g.as_slice())
                    .unwrap_or(&nav_groups_arc);
                let sidebar_with_active = render_sidebar_with_homepage(
                    sidebar_groups,
                    &page.slug,
                    homepage_slug.as_deref(),
                );
                let breadcrumbs = generate_breadcrumbs(&page.slug);
                let breadcrumb_html = render_breadcrumbs(&breadcrumbs);

                let page_title = extract_page_title(&root).unwrap_or_else(|| page.title.clone());
                let page_description = extract_page_description(&root);
                let page_layout = extract_page_layout(&root);

                let is_homepage = homepage_slug.as_deref() == Some(page.slug.as_str());
                let render_slug = if is_homepage { "" } else { &page.slug };

                let full_html = if page_layout.as_deref() == Some("landing") {
                    render_landing_page(
                        &config_arc,
                        &page_title,
                        &content_html,
                        render_slug,
                        page_description.as_deref(),
                        &assets,
                        &locale_str,
                        &i18n_state_arc,
                        &search_provider_arc,
                    )
                } else {
                    let page_nav = build_page_nav(&page.slug, &slug_index_arc, &pages_arc);
                    let git_meta = resolve_git_meta(&page.file_path);

                    let page_meta_html = render_page_meta(
                        &config_arc,
                        &page.slug,
                        &page_nav,
                        &git_meta,
                        homepage_slug.as_deref(),
                    );

                    render_page(
                        &config_arc,
                        &page_title,
                        &content_html,
                        &toc_html,
                        &sidebar_with_active,
                        &breadcrumb_html,
                        render_slug,
                        page_description.as_deref(),
                        &page_meta_html,
                        &assets,
                        &locale_str,
                        &i18n_state_arc,
                        &search_provider_arc,
                    )
                };
                let page_output = if is_homepage {
                    locale_output_dir.join("index.html")
                } else {
                    locale_output_dir.join(format!("{}.html", page.slug))
                };
                if let Some(parent) = page_output.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                        path: parent.display().to_string(),
                        source: e,
                    })?;
                }

                let minified_html = minify_html(&full_html);
                std::fs::write(&page_output, minified_html).map_err(|e| {
                    OxidocError::FileWrite {
                        path: page_output.display().to_string(),
                        source: e,
                    }
                })?;

                tracing::info!(page = %page.slug, locale = %locale_str, "Rendered");
                Ok(true)
            })
            .collect();

        let rendered_flags = results?;
        let locale_pages_rendered = rendered_flags.iter().filter(|&&b| b).count();
        pages_rendered_total += locale_pages_rendered;
    }

    for (page, content) in &page_contents {
        cache.record(&page.file_path.display().to_string(), content.as_bytes());
    }

    // Process OpenAPI specs from navigation entries
    for nav_entry in &config.routing.navigation {
        if let Some(ref spec_path) = nav_entry.openapi {
            let resolved = project_root.join(spec_path);
            let spec = openapi::load_openapi_spec(&resolved)?;
            let base_path = nav_entry.path.trim_matches('/');
            let prefix = if base_path.is_empty() {
                "api".to_string()
            } else {
                base_path.to_string()
            };
            let group_title = nav_entry
                .groups
                .first()
                .map(|g| g.group.as_str())
                .unwrap_or("API Reference");
            let api_nav = openapi::generate_api_nav_groups(
                &openapi::extract_endpoints(&spec),
                group_title,
                &prefix,
            );

            let api_ctx = openapi::ApiBuildContext {
                config: &config,
                assets: &assets,
                search_provider: &search_provider,
            };
            let api_count =
                openapi::build_api_pages(&spec, output_dir, &api_nav, &prefix, &api_ctx)?;
            pages_rendered_total += api_count;
        }
    }

    let assets_copied = copy_assets(project_root, output_dir)?;
    if assets_copied > 0 {
        tracing::info!(count = assets_copied, "Assets copied");
    }

    // Render root-level pages (homepage + extra pages at /)
    if let Some(ref root_cfg) = root_config {
        let mut root_files = vec![(&root_cfg.homepage, true)];
        for p in &root_cfg.pages {
            root_files.push((p, false));
        }
        for (rdx_file, is_homepage) in &root_files {
            let rdx_path = project_root.join(rdx_file);
            let content =
                std::fs::read_to_string(&rdx_path).map_err(|e| OxidocError::FileRead {
                    path: rdx_path.display().to_string(),
                    source: e,
                })?;
            let root = rdx_parser::parse(&content);
            check_parse_errors(&root, &rdx_path.display().to_string())?;
            let content_html = render_document(
                &root,
                &config.components.custom,
                config.project.debug_islands,
            );
            let page_title =
                extract_page_title(&root).unwrap_or_else(|| config.project.name.clone());
            let page_description = extract_page_description(&root);
            let page_layout = extract_page_layout(&root);

            let slug = if *is_homepage {
                ""
            } else {
                rdx_path.file_stem().and_then(|s| s.to_str()).unwrap_or("")
            };

            let full_html = if page_layout.as_deref() == Some("landing") {
                render_landing_page(
                    &config,
                    &page_title,
                    &content_html,
                    slug,
                    page_description.as_deref(),
                    &assets,
                    &i18n_state.default_locale,
                    &i18n_state,
                    &search_provider,
                )
            } else {
                render_page(
                    &config,
                    &page_title,
                    &content_html,
                    "",
                    "",
                    "",
                    slug,
                    page_description.as_deref(),
                    "",
                    &assets,
                    &i18n_state.default_locale,
                    &i18n_state,
                    &search_provider,
                )
            };

            let out_path = if *is_homepage {
                output_dir.join("index.html")
            } else {
                output_dir.join(format!("{slug}.html"))
            };
            let minified = minify_html(&full_html);
            std::fs::write(&out_path, minified).map_err(|e| OxidocError::FileWrite {
                path: out_path.display().to_string(),
                source: e,
            })?;
            pages_rendered_total += 1;
            if *is_homepage {
                tracing::info!("Rendered homepage");
            } else {
                tracing::info!(page = %slug, "Rendered root page");
            }
        }
    }

    generate_llms_txt(&nav_groups, output_dir)?;
    // Only generate redirect if no root homepage was rendered
    if !has_root {
        generate_index_redirect(&nav_groups, output_dir)?;
    }
    generate_seo_files(&nav_groups, &config, output_dir)?;

    // Generate category index pages for folders with children but no index.rdx
    let folder_ctx = FolderIndexContext {
        config: &config,
        assets: &assets,
        search_provider: &search_provider,
        i18n_state: &i18n_state,
        homepage_slug: homepage_slug.as_deref(),
        section_nav_map: &section_nav_map,
    };
    generate_folder_index_pages(&nav_groups, output_dir, &folder_ctx)?;

    // Generate 404 page for each locale
    for locale in &build_locales {
        let locale_output_dir = if i18n_state.is_default_locale(locale) {
            output_dir.to_path_buf()
        } else {
            output_dir.join(locale)
        };

        let not_found_html =
            render_404_page(&config, &assets, locale, &i18n_state, &search_provider);
        let not_found_minified = minify_html(&not_found_html);
        std::fs::write(locale_output_dir.join("404.html"), not_found_minified).map_err(|e| {
            OxidocError::FileWrite {
                path: locale_output_dir.join("404.html").display().to_string(),
                source: e,
            }
        })?;
    }

    // Generate redirect pages (user-configured + auto homepage redirect)
    let mut redirects = config.redirects.redirects.clone();
    if let Some(ref slug) = homepage_slug {
        redirects.push(crate::config::RedirectEntry {
            from: format!("/{slug}"),
            to: "/".to_string(),
        });
    }
    generate_redirects(&redirects, output_dir)?;

    // Save incremental cache
    cache.save(output_dir)?;

    // Generate search indices (only for built-in oxidoc provider)
    if search_provider.is_builtin() {
        crate::search_index::generate_search_index(
            &nav_groups,
            output_dir,
            &config.search,
            bundled_model,
        )?;
    }

    // Generate per-locale JSON translation bundles for Wasm islands
    if !translation_bundles.is_empty() {
        crate::i18n::generate_translation_bundles(&translation_bundles, output_dir)?;
    }

    Ok(BuildResult {
        pages_rendered: pages_rendered_total,
        output_dir: output_dir.display().to_string(),
    })
}

/// Context for generating folder index/category pages.
struct FolderIndexContext<'a> {
    config: &'a Arc<crate::config::OxidocConfig>,
    assets: &'a AssetConfig<'a>,
    search_provider: &'a Arc<SearchProvider>,
    i18n_state: &'a Arc<crate::i18n::I18nState>,
    homepage_slug: Option<&'a str>,
    section_nav_map: &'a Arc<HashMap<String, Vec<crate::crawler::NavGroup>>>,
}

/// Write a redirect HTML page at `index_path` pointing to `target_url`.
fn write_redirect_page(index_path: &Path, target_url: &str) -> Result<()> {
    if index_path.is_file() {
        return Ok(());
    }
    if let Some(parent) = index_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
            path: parent.display().to_string(),
            source: e,
        })?;
    }
    let redirect_html = format!(
        r#"<!DOCTYPE html><html><head><meta http-equiv="refresh" content="0;url=/{target_url}"><link rel="canonical" href="/{target_url}"></head><body></body></html>"#
    );
    std::fs::write(index_path, redirect_html).map_err(|e| OxidocError::FileWrite {
        path: index_path.display().to_string(),
        source: e,
    })?;
    Ok(())
}

/// Convert a hyphenated basename to title case (e.g. "getting-started" → "Getting Started").
fn title_case(basename: &str) -> String {
    basename
        .split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(ch) => {
                    let upper: String = ch.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate category index pages for folders that have child pages but no dedicated index.rdx.
/// These pages list all child pages in the folder as a proper navigable page.
fn generate_folder_index_pages(
    nav_groups: &[crate::crawler::NavGroup],
    output_dir: &Path,
    ctx: &FolderIndexContext<'_>,
) -> Result<()> {
    use std::collections::HashSet;

    let all_pages: Vec<_> = nav_groups.iter().flat_map(|g| g.pages.iter()).collect();

    let all_slugs: Vec<&str> = all_pages.iter().map(|p| p.slug.as_str()).collect();

    // Folders that have an explicit index page
    let has_index: HashSet<&str> = all_slugs
        .iter()
        .filter(|s| s.ends_with("/index"))
        .map(|s| &s[..s.len() - "/index".len()])
        .collect();

    // Collect children per folder (preserving navigation order)
    let mut folder_children: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    for page in &all_pages {
        if let Some(pos) = page.slug.rfind('/') {
            let parent = &page.slug[..pos];
            folder_children
                .entry(parent)
                .or_default()
                .push((&page.slug, &page.title));
        }
    }

    for (folder, children) in &folder_children {
        if has_index.contains(folder) {
            continue;
        }

        let index_path = output_dir.join(folder).join("index.html");
        if index_path.is_file() {
            continue;
        }

        // If the folder itself is a page (e.g., deployment.html), redirect trailing slash
        let folder_html = output_dir.join(format!("{folder}.html"));
        if folder_html.is_file() {
            write_redirect_page(&index_path, folder)?;
            continue;
        }

        // Generate a real category page
        let basename = folder.rsplit('/').next().unwrap_or(folder);
        let folder_title = title_case(basename);
        let folder_title_escaped = crate::utils::html_escape(&folder_title);

        // Build content HTML: list of child pages as cards
        let mut content_html = format!(
            "<h1 class=\"oxidoc-heading\">{folder_title_escaped}</h1><p>Browse the pages in this section:</p>"
        );
        content_html.push_str("<div class=\"oxidoc-card-grid\">");
        for (slug, title) in children {
            let slug_escaped = crate::utils::html_escape(slug);
            let title_escaped = crate::utils::html_escape(title);
            content_html.push_str(&format!(
                "<a href=\"/{slug_escaped}\" class=\"oxidoc-card\"><div class=\"oxidoc-card-title\">{title_escaped}</div></a>"
            ));
        }
        content_html.push_str("</div>");

        // Get the sidebar for this folder's section
        let sample_slug = children.first().map(|(s, _)| *s).unwrap_or(folder);
        let sidebar_groups = ctx
            .section_nav_map
            .get(sample_slug)
            .map(|g| g.as_slice())
            .unwrap_or(nav_groups);
        let sidebar_html = render_sidebar_with_homepage(sidebar_groups, "", ctx.homepage_slug);

        let full_html = render_page(
            ctx.config,
            &folder_title,
            &content_html,
            "",
            &sidebar_html,
            "",
            folder,
            Some(&format!("Pages in the {folder_title} section")),
            "",
            ctx.assets,
            "en",
            ctx.i18n_state,
            ctx.search_provider,
        );

        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        let minified = minify_html(&full_html);
        std::fs::write(&index_path, minified).map_err(|e| OxidocError::FileWrite {
            path: index_path.display().to_string(),
            source: e,
        })?;

        tracing::info!(folder = %folder, "Generated category index");
    }

    Ok(())
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
            "search-meta.bin",
        ] {
            assert!(output.join(f).exists(), "{f} should exist");
        }

        let read = |f: &str| std::fs::read_to_string(output.join(f)).unwrap();
        let css = std::fs::read_to_string(find_hashed_file(&output, "oxidoc.", ".css")).unwrap();
        assert!(css.contains("oxidoc-primary"));
        let js =
            std::fs::read_to_string(find_hashed_file(&output, "oxidoc-loader.", ".js")).unwrap();
        assert!(js.contains("oxidoc_registry.js"));

        let intro = read("intro.html");
        assert!(
            intro.contains("Introduction")
                && intro.contains("<!DOCTYPE html>")
                && intro.contains("Test Project")
        );
        assert!(read("llms.txt").contains("/intro") && read("llms.txt").contains("/setup"));
        assert!(
            read("llms-full.txt").contains("Introduction")
                && read("llms-full.txt").contains("(intro)")
        );
        assert!(read("sitemap.xml").contains("intro.html"));
        assert!(read("robots.txt").contains("User-agent: *"));
        assert!(read("404.html").contains("404") && read("404.html").contains("Not Found"));
    }

    #[test]
    fn build_site_error_and_routing() {
        // Missing config
        let tmp = tempfile::tempdir().unwrap();
        assert!(matches!(
            build_site(tmp.path(), &tmp.path().join("d")).unwrap_err(),
            OxidocError::ConfigRead { .. }
        ));

        // Missing page in routing
        let cfg = "[project]\nname = \"X\"\n\n[routing]\nnavigation = [\n  { path = \"/\", dir = \"docs\", groups = [{ group = \"G\", pages = [\"nonexistent\"] }] }\n]\n";
        let (tmp2, out2) = setup_project(cfg, &[]);
        assert!(matches!(
            build_site(tmp2.path(), &out2).unwrap_err(),
            OxidocError::PageNotFound { .. }
        ));

        // Explicit routing works
        let cfg2 = "[project]\nname = \"R\"\n\n[routing]\nnavigation = [\n  { path = \"/\", dir = \"docs\", groups = [{ group = \"Guide\", pages = [\"qs\"] }] }\n]\n";
        let (tmp3, out3) = setup_project(cfg2, &[("qs.rdx", "# QS\n\nGo!")]);
        assert_eq!(build_site(tmp3.path(), &out3).unwrap().pages_rendered, 1);
        assert!(
            std::fs::read_to_string(out3.join("qs.html"))
                .unwrap()
                .contains("Guide")
        );
    }
}
