use super::ApiEndpoint;
use crate::crawler::{NavGroup, PageEntry};
use crate::error::{OxidocError, Result};
use openapiv3::OpenAPI;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Group endpoints by their first tag (or "Untagged").
pub fn group_endpoints_by_tag(endpoints: &[ApiEndpoint]) -> BTreeMap<String, Vec<&ApiEndpoint>> {
    let mut groups: BTreeMap<String, Vec<&ApiEndpoint>> = BTreeMap::new();
    for ep in endpoints {
        let tag = ep
            .tags
            .first()
            .cloned()
            .unwrap_or_else(|| "Untagged".into());
        groups.entry(tag).or_default().push(ep);
    }
    groups
}

/// Generate NavGroups for API endpoints to merge into the site sidebar.
pub fn generate_api_nav_groups(endpoints: &[ApiEndpoint], group_title: &str) -> Vec<NavGroup> {
    let by_tag = group_endpoints_by_tag(endpoints);
    let mut nav_groups = Vec::new();

    for (tag, eps) in &by_tag {
        let title = if by_tag.len() == 1 {
            group_title.to_string()
        } else {
            format!("{group_title} — {tag}")
        };

        let pages = eps
            .iter()
            .map(|ep| {
                let slug = endpoint_slug(ep);
                let title = ep
                    .summary
                    .clone()
                    .or_else(|| ep.operation_id.clone())
                    .unwrap_or_else(|| format!("{} {}", ep.method, ep.path));
                PageEntry {
                    title,
                    slug,
                    file_path: PathBuf::new(),
                    group: Some(tag.clone()),
                }
            })
            .collect();

        nav_groups.push(NavGroup { title, pages });
    }

    nav_groups
}

/// Generate the slug for an API endpoint page.
pub fn endpoint_slug(ep: &ApiEndpoint) -> String {
    let clean_path = ep
        .path
        .trim_start_matches('/')
        .replace('/', "-")
        .replace(['{', '}'], "");
    format!("api/{}-{}", ep.method.to_lowercase(), clean_path)
}

/// Build HTML pages for all API endpoints and return the number of pages rendered.
pub fn build_api_pages(
    spec: &OpenAPI,
    output_dir: &Path,
    config: &crate::config::OxidocConfig,
    all_nav_groups: &[NavGroup],
    assets: &crate::template_assets::AssetConfig<'_>,
    search_provider: &crate::search_provider::SearchProvider,
    theme: &crate::theme::ResolvedTheme,
) -> Result<usize> {
    let endpoints = super::parser::extract_endpoints(spec);
    let mut count = 0;

    for ep in &endpoints {
        let slug = endpoint_slug(ep);
        let content_html = super::html::render_endpoint_html(ep);
        let title = ep
            .summary
            .clone()
            .or_else(|| ep.operation_id.clone())
            .unwrap_or_else(|| format!("{} {}", ep.method, ep.path));

        let toc_html = super::html::render_api_toc(ep);
        let sidebar_html = crate::template_parts::render_sidebar(all_nav_groups, &slug);
        let breadcrumbs = crate::breadcrumb::generate_breadcrumbs(&slug);
        let breadcrumb_html = crate::breadcrumb::render_breadcrumbs(&breadcrumbs);

        let i18n_state =
            crate::i18n::I18nState::from_config(&config.i18n.default_locale, &config.i18n.locales);

        let full_html = crate::template::render_page(
            config,
            &title,
            &content_html,
            &toc_html,
            &sidebar_html,
            &breadcrumb_html,
            &slug,
            None,
            "",
            assets,
            &config.i18n.default_locale,
            &i18n_state,
            search_provider,
            theme,
        );

        let page_output = output_dir.join(format!("{slug}.html"));
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

        count += 1;
        tracing::info!(page = %slug, "Rendered API endpoint");
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openapi::parser::extract_endpoints;
    use crate::openapi::test_helpers::sample_spec;

    #[test]
    fn group_by_tag() {
        let spec = sample_spec();
        let endpoints = extract_endpoints(&spec);
        let groups = group_endpoints_by_tag(&endpoints);
        assert_eq!(groups.len(), 1);
        assert!(groups.contains_key("pets"));
        assert_eq!(groups["pets"].len(), 3);
    }

    #[test]
    fn endpoint_slug_generation() {
        let ep = ApiEndpoint {
            path: "/pets/{petId}".into(),
            method: "GET".into(),
            operation_id: None,
            summary: None,
            description: None,
            tags: vec![],
            parameters: vec![],
            request_body: None,
            responses: vec![],
            deprecated: false,
        };
        assert_eq!(endpoint_slug(&ep), "api/get-pets-petId");
    }

    #[test]
    fn nav_groups_from_endpoints() {
        let spec = sample_spec();
        let endpoints = extract_endpoints(&spec);
        let nav = generate_api_nav_groups(&endpoints, "API Reference");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].title, "API Reference");
        assert_eq!(nav[0].pages.len(), 3);
    }
}
