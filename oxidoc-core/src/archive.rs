//! Version archive: snapshot docs as rkyv blobs for multi-version sites.
//!
//! `oxidoc archive v1.0` serializes all parsed pages, config, search index,
//! and OpenAPI endpoints into a single `.rdx.archive` file. During build,
//! archived versions are deserialized and rendered with the current UI.

use crate::crawler::NavGroup;
use crate::error::{OxidocError, Result};
use crate::openapi::types::ApiEndpoint;
use crate::search_index::types::LexicalIndex;
use rdx_ast::Root;
use std::path::Path;

use crate::html_inject::{inject_outdated_banner, inject_version_switcher};

/// Rewrite internal links in archived HTML to include the version prefix.
///
/// Transforms `href="/docs/intro"` → `href="/v1.0/docs/intro"` for all internal
/// links except assets (files with extensions like .css, .js, .svg, .wasm, .png).
fn rewrite_links_for_version(html: &str, version: &str) -> String {
    // Match href="/ that aren't followed by a filename with extension
    let prefix = format!("/{version}");
    let mut result = String::with_capacity(html.len() + 1024);
    let mut remaining = html;

    while let Some(pos) = remaining.find("href=\"/") {
        // Write everything up to and including href="
        result.push_str(&remaining[..pos + 6]);
        remaining = &remaining[pos + 6..];

        // Now remaining starts with the path after href="
        // Find the closing quote
        let end = remaining.find('"').unwrap_or(remaining.len());
        let path = &remaining[..end];

        // Skip rewriting if:
        // - Already version-prefixed
        // - Is an asset (last segment has a file extension)
        // - Is just "/"
        let should_rewrite = path.len() > 1
            && !path.starts_with(&prefix)
            && !path.rsplit('/').next().is_some_and(|seg| seg.contains('.'));

        if should_rewrite {
            result.push_str(&prefix);
        }
        result.push_str(path);
        remaining = &remaining[end..];
    }
    result.push_str(remaining);
    result
}

/// A complete snapshot of a documentation version.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct VersionArchive {
    /// Version label (e.g., "v1.0")
    pub version: String,
    /// Snapshot of routing config (sections, groups, sidebar structure)
    pub routing: ArchivedRouting,
    /// Project metadata snapshot
    pub project: ArchivedProject,
    /// All pages with their parsed ASTs and metadata
    pub pages: Vec<ArchivedPage>,
    /// Pre-built search index for this version
    pub search_index: LexicalIndex,
    /// Parsed OpenAPI endpoints (not raw YAML)
    pub api_endpoints: Vec<ArchivedApiEndpoint>,
    /// Pre-rendered API pages (so archived versions don't need to re-parse specs)
    pub api_pages: Vec<ArchivedApiPage>,
    /// Root config snapshot (homepage + standalone pages)
    pub root_pages: Vec<ArchivedPage>,
}

/// Snapshot of routing/navigation structure.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedRouting {
    pub sections: Vec<ArchivedSection>,
    pub header_links: Vec<ArchivedHeaderLink>,
}

/// A site section snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedSection {
    /// Base URL path (e.g., "/", "/api")
    pub path: String,
    /// Nav groups for this section's sidebar
    pub groups: Vec<ArchivedNavGroup>,
}

/// A sidebar group snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedNavGroup {
    pub title: String,
    pub pages: Vec<ArchivedNavEntry>,
}

/// A nav entry (slug + title) for sidebar rendering.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedNavEntry {
    pub slug: String,
    pub title: String,
}

/// A header link snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedHeaderLink {
    pub label: String,
    pub href: String,
}

/// Project metadata snapshot.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedProject {
    pub name: String,
    pub description: Option<String>,
}

/// A single archived page with its AST and extracted metadata.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedPage {
    pub slug: String,
    /// Which section this page belongs to (e.g., "/", "/api")
    pub section_path: String,
    /// Sidebar group name
    pub group: String,
    /// Full parsed RDX AST
    pub ast: Root,
    /// Extracted page title
    pub title: String,
    /// Extracted page description
    pub description: Option<String>,
}

/// A pre-rendered archived API page.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedApiPage {
    pub slug: String,
    pub title: String,
    /// Pre-rendered content HTML (ready to inject into the page template)
    pub rendered_html: String,
}

/// An archived OpenAPI endpoint.
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[rkyv(crate = rkyv)]
pub struct ArchivedApiEndpoint {
    pub path: String,
    pub method: String,
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub deprecated: bool,
}

impl From<&ApiEndpoint> for ArchivedApiEndpoint {
    fn from(ep: &ApiEndpoint) -> Self {
        Self {
            path: ep.path.clone(),
            method: ep.method.clone(),
            operation_id: ep.operation_id.clone(),
            summary: ep.summary.clone(),
            description: ep.description.clone(),
            tags: ep.tags.clone(),
            deprecated: ep.deprecated,
        }
    }
}

/// Create a version archive from the current project state.
pub fn create_archive(project_root: &Path, version: &str) -> Result<VersionArchive> {
    let config = crate::config::load_config(project_root)?;
    let sections = crate::crawler::discover_sections(project_root, &config)?;
    let nav_groups = crate::crawler::discover_pages(project_root, &config)?;

    // Parse all pages and extract metadata
    let mut archived_pages = Vec::new();
    for section in &sections {
        for group in &section.nav_groups {
            for page in &group.pages {
                let content = std::fs::read_to_string(&page.file_path).map_err(|e| {
                    OxidocError::FileRead {
                        path: page.file_path.display().to_string(),
                        source: e,
                    }
                })?;
                let ast = rdx_parser::parse(&content);
                let title = crate::page_extract::extract_page_title(&ast)
                    .unwrap_or_else(|| page.title.clone());
                let description = crate::page_extract::extract_page_description(&ast);

                archived_pages.push(ArchivedPage {
                    slug: page.slug.clone(),
                    section_path: section.path.clone(),
                    group: group.title.clone(),
                    ast,
                    title,
                    description,
                });
            }
        }
    }

    // Parse root pages if configured
    let mut root_pages = Vec::new();
    if let Some(ref root_cfg) = config.routing.root {
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
            let ast = rdx_parser::parse(&content);
            let title = crate::page_extract::extract_page_title(&ast)
                .unwrap_or_else(|| config.project.name.clone());
            let description = crate::page_extract::extract_page_description(&ast);
            let slug = if *is_homepage {
                String::new()
            } else {
                rdx_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            };

            root_pages.push(ArchivedPage {
                slug,
                section_path: "/".to_string(),
                group: String::new(),
                ast,
                title,
                description,
            });
        }
    }

    // Build search index
    let page_texts = crate::search_index::extract::extract_page_text(&nav_groups)?;
    let search_index = crate::search_index::lexical::build_lexical_index(&page_texts);

    // Extract OpenAPI endpoints and pre-render API pages
    let mut api_endpoints = Vec::new();
    let mut api_pages = Vec::new();
    for nav_entry in &config.routing.navigation {
        if let Some(ref spec_path) = nav_entry.openapi {
            let resolved = project_root.join(spec_path);
            let spec = crate::openapi::load_openapi_spec(&resolved)?;
            let endpoints = crate::openapi::extract_endpoints(&spec);
            let base_url = spec.servers.first().map(|s| s.url.as_str());

            let base_path = nav_entry.path.trim_matches('/');
            let prefix = if base_path.is_empty() {
                "api".to_string()
            } else {
                base_path.to_string()
            };

            // Pre-render index page
            let spec_title = spec.info.title.clone();
            let index_html = crate::openapi::html::render_api_index(&endpoints, &spec_title);
            api_pages.push(ArchivedApiPage {
                slug: format!("{prefix}/index"),
                title: spec_title.clone(),
                rendered_html: index_html,
            });

            // Pre-render each endpoint page
            for ep in &endpoints {
                let slug = crate::openapi::nav::endpoint_slug_with_prefix(ep, &prefix);
                let content_html = crate::openapi::render_endpoint_html(ep, base_url);
                let title = ep
                    .summary
                    .clone()
                    .or_else(|| ep.operation_id.clone())
                    .unwrap_or_else(|| format!("{} {}", ep.method, ep.path));
                api_pages.push(ArchivedApiPage {
                    slug,
                    title,
                    rendered_html: content_html,
                });
            }

            api_endpoints.extend(endpoints.iter().map(ArchivedApiEndpoint::from));
        }
    }

    // Build routing snapshot
    let archived_routing = ArchivedRouting {
        sections: sections
            .iter()
            .map(|s| ArchivedSection {
                path: s.path.clone(),
                groups: s
                    .nav_groups
                    .iter()
                    .map(|g| ArchivedNavGroup {
                        title: g.title.clone(),
                        pages: g
                            .pages
                            .iter()
                            .map(|p| ArchivedNavEntry {
                                slug: p.slug.clone(),
                                title: crate::page_extract::extract_page_title(&rdx_parser::parse(
                                    &std::fs::read_to_string(&p.file_path).unwrap_or_default(),
                                ))
                                .unwrap_or_else(|| p.title.clone()),
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        header_links: config
            .routing
            .header_links
            .iter()
            .map(|l| ArchivedHeaderLink {
                label: l.label.clone(),
                href: l.href.clone(),
            })
            .collect(),
    };

    Ok(VersionArchive {
        version: version.to_string(),
        routing: archived_routing,
        project: ArchivedProject {
            name: config.project.name.clone(),
            description: config.project.description.clone(),
        },
        pages: archived_pages,
        search_index,
        api_endpoints,
        api_pages,
        root_pages,
    })
}

/// Serialize a `VersionArchive` to bytes using rkyv.
pub fn serialize_archive(archive: &VersionArchive) -> Result<Vec<u8>> {
    let aligned = rkyv::to_bytes::<rkyv::rancor::Error>(archive)
        .map_err(|e| OxidocError::Search(format!("Failed to serialize version archive: {e}")))?;
    Ok(aligned.to_vec())
}

/// Deserialize a `VersionArchive` from rkyv bytes.
pub fn deserialize_archive(bytes: &[u8]) -> Result<VersionArchive> {
    rkyv::from_bytes::<VersionArchive, rkyv::rancor::Error>(bytes)
        .map_err(|e| OxidocError::Search(format!("Failed to deserialize version archive: {e}")))
}

/// Write a version archive to the archives directory.
pub fn write_archive(project_root: &Path, version: &str, archive: &VersionArchive) -> Result<()> {
    let archives_dir = project_root.join("archives");
    std::fs::create_dir_all(&archives_dir).map_err(|e| OxidocError::DirCreate {
        path: archives_dir.display().to_string(),
        source: e,
    })?;

    let archive_path = archives_dir.join(format!("{version}.rdx.archive"));
    let bytes = serialize_archive(archive)?;
    std::fs::write(&archive_path, &bytes).map_err(|e| OxidocError::FileWrite {
        path: archive_path.display().to_string(),
        source: e,
    })?;

    tracing::info!(
        version = %version,
        pages = archive.pages.len(),
        size_kb = bytes.len() / 1024,
        "Archive created"
    );

    Ok(())
}

/// Read a version archive from the archives directory.
pub fn read_archive(project_root: &Path, version: &str) -> Result<VersionArchive> {
    let archive_path = project_root
        .join("archives")
        .join(format!("{version}.rdx.archive"));
    let bytes = std::fs::read(&archive_path).map_err(|e| OxidocError::FileRead {
        path: archive_path.display().to_string(),
        source: e,
    })?;
    deserialize_archive(&bytes)
}

/// Discover all available archive versions in the project.
pub fn discover_archives(project_root: &Path) -> Vec<String> {
    let archives_dir = project_root.join("archives");
    if !archives_dir.is_dir() {
        return Vec::new();
    }

    let mut versions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&archives_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(version) = name.strip_suffix(".rdx.archive") {
                versions.push(version.to_string());
            }
        }
    }
    versions.sort_by(|a, b| b.cmp(a));
    versions
}

/// Build all pages from an archived version into `output_dir/{version}/`.
///
/// Uses the **archived** config for nav/sidebar structure but the **current** config
/// for theme, templates, and UI.
pub(crate) fn build_archived_version(
    archive: &VersionArchive,
    output_dir: &Path,
    config: &crate::config::OxidocConfig,
    assets: &crate::template_assets::AssetConfig<'_>,
    i18n_state: &crate::i18n::I18nState,
    search_provider: &crate::search_provider::SearchProvider,
    version_switcher_html: &str,
) -> Result<usize> {
    use crate::breadcrumb::{generate_breadcrumbs, render_breadcrumbs};
    use crate::minify::minify_html;
    use crate::renderer::render_document;
    use crate::template::render_page;
    use crate::template_landing::render_landing_page;
    use crate::template_parts::render_sidebar_with_homepage;
    use crate::toc::{extract_toc, render_toc};
    use std::collections::HashMap;

    let version_dir = output_dir.join(&archive.version);
    std::fs::create_dir_all(&version_dir).map_err(|e| OxidocError::DirCreate {
        path: version_dir.display().to_string(),
        source: e,
    })?;

    let nav_groups = archive_to_nav_groups(archive);

    // Build section nav map from archived sections
    let section_nav_map: HashMap<String, Vec<NavGroup>> = {
        let mut map = HashMap::new();
        for section in &archive.routing.sections {
            let section_groups: Vec<NavGroup> = section
                .groups
                .iter()
                .map(|g| NavGroup {
                    title: g.title.clone(),
                    pages: g
                        .pages
                        .iter()
                        .map(|p| crate::crawler::PageEntry {
                            title: p.title.clone(),
                            slug: p.slug.clone(),
                            file_path: std::path::PathBuf::new(),
                            group: Some(g.title.clone()),
                        })
                        .collect(),
                })
                .collect();
            for group in &section_groups {
                for page in &group.pages {
                    map.insert(page.slug.clone(), section_groups.clone());
                }
            }
        }
        map
    };

    // Build flat page list for prev/next nav
    let flat_pages: Vec<crate::crawler::PageEntry> = nav_groups
        .iter()
        .flat_map(|g| g.pages.iter().cloned())
        .collect();
    let slug_index: HashMap<String, usize> = flat_pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.slug.clone(), i))
        .collect();
    let title_map = crate::page_extract::build_title_map(&flat_pages);

    let mut pages_rendered = 0;

    for archived_page in &archive.pages {
        let content_html = render_document(
            &archived_page.ast,
            &config.components.custom,
            config.project.debug_islands,
        );
        let toc_entries = extract_toc(&archived_page.ast);
        let toc_html = render_toc(&toc_entries);

        let sidebar_groups = section_nav_map
            .get(&archived_page.slug)
            .map(|g| g.as_slice())
            .unwrap_or(&nav_groups);
        let sidebar_html = render_sidebar_with_homepage(sidebar_groups, &archived_page.slug, None);

        let breadcrumbs = generate_breadcrumbs(&archived_page.slug);
        let breadcrumb_html = render_breadcrumbs(&breadcrumbs);

        let page_layout = crate::page_extract::extract_page_layout(&archived_page.ast);

        let full_html = if page_layout.as_deref() == Some("landing") {
            render_landing_page(
                config,
                &archived_page.title,
                &content_html,
                &archived_page.slug,
                archived_page.description.as_deref(),
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                false,
            )
        } else {
            let page_nav = crate::page_extract::build_page_nav(
                &archived_page.slug,
                &slug_index,
                &flat_pages,
                &title_map,
            );
            let git_meta = crate::template_parts::PageGitMeta::default();
            let page_meta_html = crate::template_parts::render_page_meta(
                config,
                &archived_page.slug,
                &page_nav,
                &git_meta,
                None,
            );

            render_page(
                config,
                &archived_page.title,
                &content_html,
                &toc_html,
                &sidebar_html,
                &breadcrumb_html,
                &archived_page.slug,
                archived_page.description.as_deref(),
                &page_meta_html,
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                false,
            )
        };

        let page_output = if archived_page.slug.is_empty() {
            version_dir.join("index.html")
        } else {
            version_dir.join(format!("{}.html", archived_page.slug))
        };

        if let Some(parent) = page_output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        let full_html = rewrite_links_for_version(&full_html, &archive.version);
        let full_html = inject_outdated_banner(&full_html, &archive.version);
        let full_html = inject_version_switcher(&full_html, version_switcher_html);
        let minified = minify_html(&full_html);
        std::fs::write(&page_output, minified).map_err(|e| OxidocError::FileWrite {
            path: page_output.display().to_string(),
            source: e,
        })?;

        pages_rendered += 1;
    }

    // Render root pages for this version
    for root_page in &archive.root_pages {
        let content_html = render_document(
            &root_page.ast,
            &config.components.custom,
            config.project.debug_islands,
        );
        let page_layout = crate::page_extract::extract_page_layout(&root_page.ast);

        let full_html = if page_layout.as_deref() == Some("landing") {
            render_landing_page(
                config,
                &root_page.title,
                &content_html,
                &root_page.slug,
                root_page.description.as_deref(),
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                false,
            )
        } else {
            render_page(
                config,
                &root_page.title,
                &content_html,
                "",
                "",
                "",
                &root_page.slug,
                root_page.description.as_deref(),
                "",
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                false,
            )
        };

        let out_path = if root_page.slug.is_empty() {
            version_dir.join("index.html")
        } else {
            version_dir.join(format!("{}.html", root_page.slug))
        };

        let full_html = rewrite_links_for_version(&full_html, &archive.version);
        let full_html = inject_outdated_banner(&full_html, &archive.version);
        let full_html = inject_version_switcher(&full_html, version_switcher_html);
        let minified = minify_html(&full_html);
        std::fs::write(&out_path, minified).map_err(|e| OxidocError::FileWrite {
            path: out_path.display().to_string(),
            source: e,
        })?;

        pages_rendered += 1;
    }

    // Render archived API pages
    for api_page in &archive.api_pages {
        let sidebar_groups = section_nav_map
            .get(&api_page.slug)
            .map(|g| g.as_slice())
            .unwrap_or(&nav_groups);
        let sidebar_html = render_sidebar_with_homepage(sidebar_groups, &api_page.slug, None);
        let breadcrumbs = generate_breadcrumbs(&api_page.slug);
        let breadcrumb_html = render_breadcrumbs(&breadcrumbs);

        let full_html = render_page(
            config,
            &api_page.title,
            &api_page.rendered_html,
            "",
            &sidebar_html,
            &breadcrumb_html,
            &api_page.slug,
            None,
            "",
            assets,
            &i18n_state.default_locale,
            i18n_state,
            search_provider,
            false,
        );

        let page_output = version_dir.join(format!("{}.html", api_page.slug));
        if let Some(parent) = page_output.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        let full_html = rewrite_links_for_version(&full_html, &archive.version);
        let full_html = inject_outdated_banner(&full_html, &archive.version);
        let full_html = inject_version_switcher(&full_html, version_switcher_html);
        let minified = minify_html(&full_html);
        std::fs::write(&page_output, minified).map_err(|e| OxidocError::FileWrite {
            path: page_output.display().to_string(),
            source: e,
        })?;

        pages_rendered += 1;
    }

    // Write version-scoped search index
    crate::search_index::lexical::write_lexical_index(&archive.search_index, &version_dir)?;

    // Generate 404 page for this version
    {
        use crate::minify::minify_html;
        let not_found_html = crate::template_404::render_404_page(
            config,
            assets,
            &i18n_state.default_locale,
            i18n_state,
            search_provider,
        );
        let not_found_html = inject_version_switcher(&not_found_html, version_switcher_html);
        let not_found_minified = minify_html(&not_found_html);
        let not_found_path = version_dir.join("404.html");
        std::fs::write(&not_found_path, not_found_minified).map_err(|e| {
            OxidocError::FileWrite {
                path: not_found_path.display().to_string(),
                source: e,
            }
        })?;
    }

    tracing::info!(
        version = %archive.version,
        pages = pages_rendered,
        "Rendered archived version"
    );

    Ok(pages_rendered)
}

/// Convert an archived version's nav structure back to `NavGroup`s for rendering.
pub fn archive_to_nav_groups(archive: &VersionArchive) -> Vec<NavGroup> {
    let mut nav_groups = Vec::new();
    for section in &archive.routing.sections {
        for group in &section.groups {
            nav_groups.push(NavGroup {
                title: group.title.clone(),
                pages: group
                    .pages
                    .iter()
                    .map(|p| crate::crawler::PageEntry {
                        title: p.title.clone(),
                        slug: p.slug.clone(),
                        file_path: std::path::PathBuf::new(), // No file path for archived pages
                        group: Some(group.title.clone()),
                    })
                    .collect(),
            });
        }
    }
    nav_groups
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Unit tests for inject_version_switcher ---

    #[test]
    fn inject_version_switcher_empty() {
        let html = "<header>Logo</header><main>Content</main>";
        let result = inject_version_switcher(html, "");
        assert_eq!(result, html);
    }

    #[test]
    fn inject_version_switcher_inserts_before_closing_header() {
        let html = "<header>Logo</header><main>Content</main>";
        let switcher = r#"<div class="oxidoc-version-switcher">v1</div>"#;
        let result = inject_version_switcher(html, switcher);
        assert!(result.contains(&format!("{switcher}</header>")));
        // Only injects once
        assert_eq!(result.matches("oxidoc-version-switcher").count(), 1);
    }

    #[test]
    fn inject_version_switcher_no_header() {
        let html = "<main>No header here</main>";
        let result = inject_version_switcher(html, "<div>switcher</div>");
        // No </header> to inject into, html unchanged
        assert_eq!(result, html);
    }

    // --- Unit tests for ArchivedApiEndpoint::from ---

    #[test]
    fn archived_api_endpoint_from_conversion() {
        let ep = ApiEndpoint {
            path: "/users".to_string(),
            method: "GET".to_string(),
            operation_id: Some("listUsers".to_string()),
            summary: Some("List all users".to_string()),
            description: None,
            tags: vec!["users".to_string()],
            parameters: vec![],
            request_body: None,
            responses: vec![],
            deprecated: true,
        };
        let archived = ArchivedApiEndpoint::from(&ep);
        assert_eq!(archived.path, "/users");
        assert_eq!(archived.method, "GET");
        assert_eq!(archived.operation_id, Some("listUsers".to_string()));
        assert_eq!(archived.tags, vec!["users"]);
        assert!(archived.deprecated);
    }

    // --- Unit tests for archive_to_nav_groups ---

    #[test]
    fn archive_to_nav_groups_conversion() {
        let archive = VersionArchive {
            version: "v1.0".to_string(),
            routing: ArchivedRouting {
                sections: vec![ArchivedSection {
                    path: "/".to_string(),
                    groups: vec![ArchivedNavGroup {
                        title: "Guide".to_string(),
                        pages: vec![
                            ArchivedNavEntry {
                                slug: "intro".to_string(),
                                title: "Introduction".to_string(),
                            },
                            ArchivedNavEntry {
                                slug: "setup".to_string(),
                                title: "Setup".to_string(),
                            },
                        ],
                    }],
                }],
                header_links: vec![],
            },
            project: ArchivedProject {
                name: "Test".to_string(),
                description: None,
            },
            pages: vec![],
            search_index: LexicalIndex {
                postings: std::collections::HashMap::new(),
                documents: vec![],
            },
            api_endpoints: vec![],
            api_pages: vec![],
            root_pages: vec![],
        };

        let nav = archive_to_nav_groups(&archive);
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].title, "Guide");
        assert_eq!(nav[0].pages.len(), 2);
        assert_eq!(nav[0].pages[0].slug, "intro");
        // file_path should be empty for archived pages
        assert_eq!(nav[0].pages[0].file_path, std::path::PathBuf::new());
        assert_eq!(nav[0].pages[0].group, Some("Guide".to_string()));
    }

    #[test]
    fn archive_to_nav_groups_multiple_sections() {
        let archive = VersionArchive {
            version: "v1.0".to_string(),
            routing: ArchivedRouting {
                sections: vec![
                    ArchivedSection {
                        path: "/".to_string(),
                        groups: vec![ArchivedNavGroup {
                            title: "Docs".to_string(),
                            pages: vec![ArchivedNavEntry {
                                slug: "intro".to_string(),
                                title: "Intro".to_string(),
                            }],
                        }],
                    },
                    ArchivedSection {
                        path: "/api".to_string(),
                        groups: vec![ArchivedNavGroup {
                            title: "API".to_string(),
                            pages: vec![ArchivedNavEntry {
                                slug: "api/users".to_string(),
                                title: "Users".to_string(),
                            }],
                        }],
                    },
                ],
                header_links: vec![],
            },
            project: ArchivedProject {
                name: "Test".to_string(),
                description: None,
            },
            pages: vec![],
            search_index: LexicalIndex {
                postings: std::collections::HashMap::new(),
                documents: vec![],
            },
            api_endpoints: vec![],
            api_pages: vec![],
            root_pages: vec![],
        };

        let nav = archive_to_nav_groups(&archive);
        assert_eq!(nav.len(), 2);
        assert_eq!(nav[0].title, "Docs");
        assert_eq!(nav[1].title, "API");
        assert_eq!(nav[1].pages[0].slug, "api/users");
    }

    // --- Unit tests for discover_archives ---

    #[test]
    fn discover_archives_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(discover_archives(tmp.path()).is_empty());
    }

    #[test]
    fn discover_archives_ignores_non_archive_files() {
        let tmp = tempfile::tempdir().unwrap();
        let archives_dir = tmp.path().join("archives");
        std::fs::create_dir(&archives_dir).unwrap();
        std::fs::write(archives_dir.join("v1.0.rdx.archive"), b"data").unwrap();
        std::fs::write(archives_dir.join("notes.txt"), "ignore me").unwrap();
        std::fs::write(archives_dir.join("v2.0.rdx.archive"), b"data").unwrap();

        let versions = discover_archives(tmp.path());
        assert_eq!(versions, vec!["v2.0", "v1.0"]);
    }

    // --- Unit tests for serialize/deserialize ---

    #[test]
    fn roundtrip_archive_preserves_all_fields() {
        let archive = VersionArchive {
            version: "v3.0".to_string(),
            routing: ArchivedRouting {
                sections: vec![ArchivedSection {
                    path: "/docs".to_string(),
                    groups: vec![ArchivedNavGroup {
                        title: "Guide".to_string(),
                        pages: vec![ArchivedNavEntry {
                            slug: "quickstart".to_string(),
                            title: "Quick Start".to_string(),
                        }],
                    }],
                }],
                header_links: vec![ArchivedHeaderLink {
                    label: "GitHub".to_string(),
                    href: "https://github.com".to_string(),
                }],
            },
            project: ArchivedProject {
                name: "MyProject".to_string(),
                description: Some("A project".to_string()),
            },
            pages: vec![ArchivedPage {
                slug: "quickstart".to_string(),
                section_path: "/docs".to_string(),
                group: "Guide".to_string(),
                ast: rdx_parser::parse("# Quick Start\n\nHello."),
                title: "Quick Start".to_string(),
                description: Some("Get started quickly".to_string()),
            }],
            search_index: LexicalIndex {
                postings: std::collections::HashMap::new(),
                documents: vec![],
            },
            api_endpoints: vec![ArchivedApiEndpoint {
                path: "/api/v1/users".to_string(),
                method: "POST".to_string(),
                operation_id: Some("createUser".to_string()),
                summary: Some("Create user".to_string()),
                description: None,
                tags: vec!["users".to_string()],
                deprecated: false,
            }],
            api_pages: vec![],
            root_pages: vec![],
        };

        let bytes = serialize_archive(&archive).unwrap();
        let restored = deserialize_archive(&bytes).unwrap();

        assert_eq!(restored.version, "v3.0");
        assert_eq!(restored.routing.sections.len(), 1);
        assert_eq!(restored.routing.sections[0].path, "/docs");
        assert_eq!(restored.routing.header_links.len(), 1);
        assert_eq!(restored.routing.header_links[0].label, "GitHub");
        assert_eq!(restored.project.name, "MyProject");
        assert_eq!(restored.project.description, Some("A project".to_string()));
        assert_eq!(restored.pages.len(), 1);
        assert_eq!(restored.pages[0].title, "Quick Start");
        assert_eq!(
            restored.pages[0].description,
            Some("Get started quickly".to_string())
        );
        assert_eq!(restored.api_endpoints.len(), 1);
        assert_eq!(restored.api_endpoints[0].method, "POST");
    }

    // --- Unit tests for create_archive with routing config ---

    #[test]
    fn create_archive_with_routing_sections() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        let lib = root.join("lib");
        std::fs::create_dir(&docs).unwrap();
        std::fs::create_dir(&lib).unwrap();
        std::fs::write(
            root.join("oxidoc.toml"),
            r#"[project]
name = "Multi-Section"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Guide", pages = ["intro"] }] },
  { path = "/lib", dir = "lib", groups = [{ group = "Library", pages = ["api"] }] },
]
"#,
        )
        .unwrap();
        std::fs::write(docs.join("intro.rdx"), "# Intro\n\nDocs intro.").unwrap();
        std::fs::write(lib.join("api.rdx"), "# API\n\nLibrary API.").unwrap();

        let archive = create_archive(root, "v1.0").unwrap();
        assert_eq!(archive.pages.len(), 2);
        assert_eq!(archive.routing.sections.len(), 2);
        assert_eq!(archive.routing.sections[0].path, "/");
        assert_eq!(archive.routing.sections[1].path, "/lib");

        // Pages should have correct section paths
        let intro = archive.pages.iter().find(|p| p.slug == "intro").unwrap();
        assert_eq!(intro.section_path, "/");
        let api = archive.pages.iter().find(|p| p.slug == "lib/api").unwrap();
        assert_eq!(api.section_path, "/lib");
    }

    // --- Unit tests for create_archive with root pages ---

    #[test]
    fn create_archive_with_root_pages() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(
            root.join("oxidoc.toml"),
            r#"[project]
name = "WithRoot"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Docs", pages = ["page"] }] },
]

[routing.root]
homepage = "home.rdx"
pages = ["about.rdx"]
"#,
        )
        .unwrap();
        std::fs::write(docs.join("page.rdx"), "# Page\n\nContent.").unwrap();
        std::fs::write(root.join("home.rdx"), "# Home\n\nWelcome!").unwrap();
        std::fs::write(root.join("about.rdx"), "# About\n\nAbout us.").unwrap();

        let archive = create_archive(root, "v1.0").unwrap();
        assert_eq!(archive.pages.len(), 1);
        assert_eq!(archive.root_pages.len(), 2);

        let home = archive
            .root_pages
            .iter()
            .find(|p| p.slug.is_empty())
            .unwrap();
        assert_eq!(home.title, "Home");

        let about = archive
            .root_pages
            .iter()
            .find(|p| p.slug == "about")
            .unwrap();
        assert_eq!(about.title, "About");
    }

    // --- Unit tests for write_archive/read_archive roundtrip ---

    #[test]
    fn write_and_read_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let docs = root.join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(root.join("oxidoc.toml"), "[project]\nname = \"Test\"\n").unwrap();
        std::fs::write(docs.join("page.rdx"), "# Page\n\nContent.").unwrap();

        let archive = create_archive(root, "v2.0").unwrap();
        write_archive(root, "v2.0", &archive).unwrap();

        let versions = discover_archives(root);
        assert_eq!(versions, vec!["v2.0"]);

        let restored = read_archive(root, "v2.0").unwrap();
        assert_eq!(restored.version, "v2.0");
        assert_eq!(restored.pages.len(), 1);
    }
}
