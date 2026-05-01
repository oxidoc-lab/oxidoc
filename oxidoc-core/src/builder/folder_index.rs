use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::error::{OxidocError, Result};
use crate::html_inject::inject_version_switcher;
use crate::minify::minify_html;
use crate::template::render_page;
use crate::template_assets::AssetConfig;
use crate::template_parts::render_sidebar_with_homepage;

/// Context for generating folder index/category pages.
pub(super) struct FolderIndexContext<'a> {
    pub(super) config: &'a Arc<crate::config::OxidocConfig>,
    pub(super) assets: &'a AssetConfig<'a>,
    pub(super) search_provider: &'a Arc<crate::search_provider::SearchProvider>,
    pub(super) i18n_state: &'a Arc<crate::i18n::I18nState>,
    pub(super) homepage_slug: Option<&'a str>,
    pub(super) section_nav_map: &'a Arc<HashMap<String, Vec<crate::crawler::NavGroup>>>,
    pub(super) version_switcher_html: &'a str,
}

/// Write a redirect HTML page at `index_path` pointing to `target_url`.
pub(super) fn write_redirect_page(index_path: &Path, target_url: &str) -> Result<()> {
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
pub(super) fn title_case(basename: &str) -> String {
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
pub(super) fn generate_folder_index_pages(
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

    // Collect direct page children per folder (preserving navigation order)
    let mut folder_children: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();
    // Collect direct sub-folder children per folder (preserving first-encounter order)
    let mut folder_subfolders: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut seen_subfolder: HashSet<(&str, &str)> = HashSet::new();
    for page in &all_pages {
        // Walk every ancestor folder: for slug "a/b/c", parents are "a/b", "a".
        let mut rest = page.slug.as_str();
        let mut child_path = page.slug.as_str();
        while let Some(pos) = rest.rfind('/') {
            let parent = &rest[..pos];
            if rest == page.slug {
                // immediate parent: register page as direct child
                folder_children
                    .entry(parent)
                    .or_default()
                    .push((&page.slug, &page.title));
            } else {
                // ancestor: register the intermediate folder as a sub-folder card
                if seen_subfolder.insert((parent, child_path)) {
                    folder_subfolders
                        .entry(parent)
                        .or_default()
                        .push(child_path);
                }
            }
            child_path = parent;
            rest = parent;
        }
    }

    // Union of folders that have either page children or sub-folder children
    let mut all_folders: HashSet<&str> = folder_children.keys().copied().collect();
    all_folders.extend(folder_subfolders.keys().copied());

    for folder in &all_folders {
        let folder = *folder;
        let empty_pages: Vec<(&str, &str)> = Vec::new();
        let children = folder_children.get(folder).unwrap_or(&empty_pages);
        let empty_subs: Vec<&str> = Vec::new();
        let subfolders = folder_subfolders.get(folder).unwrap_or(&empty_subs);
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
        for sub_slug in subfolders {
            let sub_basename = sub_slug.rsplit('/').next().unwrap_or(sub_slug);
            let sub_title = title_case(sub_basename);
            let slug_escaped = crate::utils::html_escape(sub_slug);
            let title_escaped = crate::utils::html_escape(&sub_title);
            content_html.push_str(&format!(
                "<a href=\"/{slug_escaped}/\" class=\"oxidoc-card\"><div class=\"oxidoc-card-title\">{title_escaped}</div></a>"
            ));
        }
        content_html.push_str("</div>");

        // Get the sidebar for this folder's section
        let sample_slug = children
            .first()
            .map(|(s, _)| *s)
            .or_else(|| {
                // Find any descendant page slug under this folder for sidebar lookup
                all_pages
                    .iter()
                    .find(|p| {
                        p.slug.starts_with(folder)
                            && p.slug.as_bytes().get(folder.len()) == Some(&b'/')
                    })
                    .map(|p| p.slug.as_str())
            })
            .unwrap_or(folder);
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
            false,
        );

        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                path: parent.display().to_string(),
                source: e,
            })?;
        }

        let full_html = inject_version_switcher(&full_html, ctx.version_switcher_html);
        let minified = minify_html(&full_html);
        std::fs::write(&index_path, minified).map_err(|e| OxidocError::FileWrite {
            path: index_path.display().to_string(),
            source: e,
        })?;

        tracing::info!(folder = %folder, "Generated category index");
    }

    Ok(())
}
