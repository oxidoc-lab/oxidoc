use crate::config::RedirectEntry;
use crate::crawler::NavGroup;
use crate::error::{OxidocError, Result};
use std::path::Path;

/// Generate `llms.txt` and `llms-full.txt` for AI/RAG consumption.
pub fn generate_llms_txt(nav_groups: &[NavGroup], output_dir: &Path) -> Result<()> {
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
