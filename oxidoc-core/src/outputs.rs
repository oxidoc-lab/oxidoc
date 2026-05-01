use crate::config::{OxidocConfig, RedirectEntry};
use crate::crawler::NavGroup;
use crate::error::{OxidocError, Result};
use std::path::Path;

/// Generate all SEO-related files (sitemap, robots.txt, feed).
pub fn generate_seo_files(
    nav_groups: &[NavGroup],
    config: &OxidocConfig,
    output_dir: &Path,
) -> Result<()> {
    let base_url = config.project.base_url.as_deref().unwrap_or("/");
    crate::sitemap::generate_sitemap(nav_groups, base_url, output_dir)?;
    crate::sitemap::generate_robots_txt(base_url, output_dir)?;
    let description = config
        .project
        .description
        .as_deref()
        .unwrap_or("Documentation");
    crate::feed::generate_feed(
        nav_groups,
        &config.project.name,
        base_url,
        description,
        output_dir,
    )
}

/// Generate an index.html that redirects to the first page.
pub fn generate_index_redirect(nav_groups: &[NavGroup], output_dir: &Path) -> Result<()> {
    let first_slug = nav_groups
        .iter()
        .flat_map(|g| g.pages.first())
        .map(|p| p.slug.as_str())
        .next();

    if let Some(slug) = first_slug {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=/{slug}"></head>
<body><a href="/{slug}">Redirecting...</a></body>
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

/// Generate redirect HTML files for configured redirects.
pub fn generate_redirects(redirects: &[RedirectEntry], output_dir: &Path) -> Result<()> {
    for redirect in redirects {
        let from_path = redirect.from.trim_start_matches('/').replace("/", "-");
        let filename = if from_path.is_empty() {
            "index.html".to_string()
        } else {
            format!("{}.html", from_path)
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url={}"></head>
<body><a href="{}">Redirecting...</a></body>
</html>"#,
            crate::utils::html_escape(&redirect.to),
            crate::utils::html_escape(&redirect.to)
        );

        let redirect_path = output_dir.join(&filename);
        std::fs::write(&redirect_path, html).map_err(|e| OxidocError::FileWrite {
            path: redirect_path.display().to_string(),
            source: e,
        })?;
    }

    Ok(())
}
