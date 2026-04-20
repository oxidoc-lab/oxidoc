use crate::crawler::NavGroup;
use crate::error::{OxidocError, Result};
use std::fmt::Write;
use std::path::Path;

/// Generate a sitemap.xml file with all page URLs.
pub fn generate_sitemap(nav_groups: &[NavGroup], base_url: &str, output_dir: &Path) -> Result<()> {
    let mut xml = String::from(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"##,
    );
    xml.push('\n');

    for group in nav_groups {
        for page in &group.pages {
            let url = format!("{}/{}", base_url.trim_end_matches('/'), page.slug);
            let _ = writeln!(xml, r##"  <url><loc>{}</loc></url>"##, html_escape(&url));
        }
    }

    xml.push_str("\n</urlset>\n");

    let sitemap_path = output_dir.join("sitemap.xml");
    std::fs::write(&sitemap_path, xml).map_err(|e| OxidocError::FileWrite {
        path: sitemap_path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

/// Generate a robots.txt file that allows all crawling and references sitemap.
pub fn generate_robots_txt(base_url: &str, output_dir: &Path) -> Result<()> {
    let sitemap_url = if base_url.ends_with('/') {
        format!("{}sitemap.xml", base_url)
    } else {
        format!("{}/sitemap.xml", base_url)
    };

    let content = format!("User-agent: *\nAllow: /\nSitemap: {}\n", sitemap_url);

    let robots_path = output_dir.join("robots.txt");
    std::fs::write(&robots_path, content).map_err(|e| OxidocError::FileWrite {
        path: robots_path.display().to_string(),
        source: e,
    })?;

    Ok(())
}

use crate::utils::html_escape;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_nav_groups() -> Vec<NavGroup> {
        vec![NavGroup {
            title: "Getting Started".into(),
            pages: vec![
                crate::crawler::PageEntry {
                    title: "Intro".into(),
                    short_title: "Intro".into(),
                    slug: "intro".into(),
                    file_path: PathBuf::new(),
                    group: None,
                },
                crate::crawler::PageEntry {
                    title: "Setup".into(),
                    short_title: "Setup".into(),
                    slug: "setup".into(),
                    file_path: PathBuf::new(),
                    group: None,
                },
            ],
        }]
    }

    #[test]
    fn generate_sitemap_creates_valid_xml() {
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path();
        let nav_groups = test_nav_groups();

        generate_sitemap(&nav_groups, "https://example.com", output).unwrap();

        let content = std::fs::read_to_string(output.join("sitemap.xml")).unwrap();
        assert!(content.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(content.contains("<urlset"));
        assert!(content.contains("https://example.com/intro"));
        assert!(content.contains("https://example.com/setup"));
        assert!(
            !content.contains(".html"),
            "sitemap must not contain .html extensions"
        );
        assert!(content.contains("</urlset>"));
    }

    #[test]
    fn generate_sitemap_handles_trailing_slash() {
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path();
        let nav_groups = test_nav_groups();

        generate_sitemap(&nav_groups, "https://example.com/", output).unwrap();

        let content = std::fs::read_to_string(output.join("sitemap.xml")).unwrap();
        assert!(content.contains("https://example.com/intro"));
        assert!(!content.contains("https://example.com//intro"));
    }

    #[test]
    fn generate_robots_txt_allows_all() {
        let tmp = tempfile::tempdir().unwrap();
        let output = tmp.path();

        generate_robots_txt("https://example.com", output).unwrap();

        let content = std::fs::read_to_string(output.join("robots.txt")).unwrap();
        assert!(content.contains("User-agent: *"));
        assert!(content.contains("Allow: /"));
        assert!(content.contains("Sitemap: https://example.com/sitemap.xml"));
    }
}
