use std::path::Path;

use crate::error::{OxidocError, Result};
use crate::html_inject::inject_version_switcher;
use crate::minify::minify_html;
use crate::page_extract::{
    check_parse_errors, extract_page_description, extract_page_layout, extract_page_title,
};
use crate::renderer::render_document;
use crate::template::render_page;
use crate::template_assets::AssetConfig;
use crate::template_landing::render_landing_page;

#[allow(clippy::too_many_arguments)]
pub(super) fn build_root_pages(
    root_cfg: &crate::config::RootConfig,
    project_root: &Path,
    output_dir: &Path,
    config: &crate::config::OxidocConfig,
    assets: &AssetConfig<'_>,
    i18n_state: &crate::i18n::I18nState,
    search_provider: &crate::search_provider::SearchProvider,
    version_switcher_html: &str,
) -> Result<usize> {
    let mut root_files = vec![(&root_cfg.homepage, true)];
    for p in &root_cfg.pages {
        root_files.push((p, false));
    }

    let mut count = 0;

    for (rdx_file, is_homepage) in &root_files {
        let rdx_path = project_root.join(rdx_file);
        let content = std::fs::read_to_string(&rdx_path).map_err(|e| OxidocError::FileRead {
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
                config,
                &page_title,
                &content_html,
                slug,
                page_description.as_deref(),
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                *is_homepage,
            )
        } else {
            render_page(
                config,
                &page_title,
                &content_html,
                "",
                "",
                "",
                slug,
                page_description.as_deref(),
                "",
                assets,
                &i18n_state.default_locale,
                i18n_state,
                search_provider,
                *is_homepage,
            )
        };

        let out_path = if *is_homepage {
            output_dir.join("index.html")
        } else {
            output_dir.join(slug).join("index.html")
        };
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
                path: parent.display().to_string(),
                source: e,
            })?;
        }
        let full_html = inject_version_switcher(&full_html, version_switcher_html);
        let minified = minify_html(&full_html);
        std::fs::write(&out_path, minified).map_err(|e| OxidocError::FileWrite {
            path: out_path.display().to_string(),
            source: e,
        })?;
        count += 1;
        if *is_homepage {
            tracing::info!("Rendered homepage");
        } else {
            tracing::info!(page = %slug, "Rendered root page");
        }
    }

    Ok(count)
}
