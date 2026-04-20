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
pub fn generate_api_nav_groups(
    endpoints: &[ApiEndpoint],
    group_title: &str,
    prefix: &str,
) -> Vec<NavGroup> {
    let by_tag = group_endpoints_by_tag(endpoints);
    let mut nav_groups = Vec::new();

    for (tag, eps) in &by_tag {
        let title = if by_tag.len() == 1 {
            group_title.to_string()
        } else {
            tag.to_string()
        };

        let pages = eps
            .iter()
            .map(|ep| {
                let slug = endpoint_slug_with_prefix(ep, prefix);
                let title = ep
                    .summary
                    .clone()
                    .or_else(|| ep.operation_id.clone())
                    .unwrap_or_else(|| format!("{} {}", ep.method, ep.path));
                PageEntry {
                    short_title: title.clone(),
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

/// Generate the slug for an API endpoint page with a custom prefix.
pub fn endpoint_slug_with_prefix(ep: &ApiEndpoint, prefix: &str) -> String {
    let clean_path = ep
        .path
        .trim_start_matches('/')
        .replace('/', "-")
        .replace(['{', '}'], "");
    format!("{}/{}-{}", prefix, ep.method.to_lowercase(), clean_path)
}

/// Generate the slug for an API endpoint page (default "api" prefix).
pub fn endpoint_slug(ep: &ApiEndpoint) -> String {
    endpoint_slug_with_prefix(ep, "api")
}

/// Shared rendering context passed to API page builders.
pub struct ApiBuildContext<'a> {
    pub config: &'a crate::config::OxidocConfig,
    pub assets: &'a crate::template_assets::AssetConfig<'a>,
    pub search_provider: &'a crate::search_provider::SearchProvider,
}

/// Build HTML pages for all API endpoints and return the number of pages rendered.
///
/// The API section is self-contained: it gets its own sidebar (only API nav groups)
/// and its own index page at `/api/`.
pub fn build_api_pages(
    spec: &OpenAPI,
    output_dir: &Path,
    api_nav_groups: &[NavGroup],
    prefix: &str,
    ctx: &ApiBuildContext<'_>,
) -> Result<usize> {
    let config = ctx.config;
    let endpoints = super::parser::extract_endpoints(spec);
    let base_url = spec.servers.first().map(|s| s.url.as_str());
    let mut count = 0;

    let i18n_state =
        crate::i18n::I18nState::from_config(&config.i18n.default_locale, &config.i18n.locales);

    // Generate index page at /{prefix}/
    let spec_title = spec.info.title.as_str();
    let index_html = super::html::render_api_index(&endpoints, spec_title);
    let index_slug = format!("{prefix}/index");
    let index_sidebar = crate::template_parts::render_sidebar(api_nav_groups, &index_slug);
    let index_full = crate::template::render_page(
        config,
        spec_title,
        &index_html,
        "",
        &index_sidebar,
        "",
        prefix,
        None,
        "",
        ctx.assets,
        &config.i18n.default_locale,
        &i18n_state,
        ctx.search_provider,
        false,
    );
    let section_dir = output_dir.join(prefix);
    std::fs::create_dir_all(&section_dir).map_err(|e| OxidocError::DirCreate {
        path: section_dir.display().to_string(),
        source: e,
    })?;
    std::fs::write(section_dir.join("index.html"), index_full).map_err(|e| {
        OxidocError::FileWrite {
            path: section_dir.join("index.html").display().to_string(),
            source: e,
        }
    })?;
    count += 1;
    tracing::info!(page = %format!("{prefix}/index"), "Rendered API index");

    for ep in &endpoints {
        let slug = endpoint_slug_with_prefix(ep, prefix);
        let content_html = super::html::render_endpoint_html(ep, base_url);
        let title = ep
            .summary
            .clone()
            .or_else(|| ep.operation_id.clone())
            .unwrap_or_else(|| format!("{} {}", ep.method, ep.path));

        let sidebar_html = crate::template_parts::render_sidebar(api_nav_groups, &slug);
        let breadcrumbs = crate::breadcrumb::generate_breadcrumbs(&slug);
        let breadcrumb_html = crate::breadcrumb::render_breadcrumbs(&breadcrumbs);

        let full_html = crate::template::render_page(
            config,
            &title,
            &content_html,
            "",
            &sidebar_html,
            &breadcrumb_html,
            &slug,
            None,
            "",
            ctx.assets,
            &config.i18n.default_locale,
            &i18n_state,
            ctx.search_provider,
            false,
        );

        let page_output = output_dir.join(&slug).join("index.html");
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
        let nav = generate_api_nav_groups(&endpoints, "API Reference", "api");
        assert_eq!(nav.len(), 1);
        assert_eq!(nav[0].title, "API Reference");
        assert_eq!(nav[0].pages.len(), 3);
    }
}
