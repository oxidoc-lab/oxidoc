use crate::config::OxidocConfig;
use crate::crawler::NavGroup;
use std::fmt::Write;

/// Wrap rendered page content in a full HTML document.
pub fn render_page(
    config: &OxidocConfig,
    title: &str,
    content_html: &str,
    toc_html: &str,
    sidebar_html: &str,
    breadcrumb_html: &str,
    active_slug: &str,
) -> String {
    let project_name = &config.project.name;
    let primary_color = &config.theme.primary;
    let page_title = if title.is_empty() {
        project_name.clone()
    } else {
        format!("{title} — {project_name}")
    };

    let base_url = config.project.base_url.as_deref().unwrap_or("/");

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{page_title}</title>
    <meta name="generator" content="oxidoc">
    <link rel="canonical" href="{base_url}{active_slug}">
    <style>:root {{ --oxidoc-primary: {primary_color}; }}</style>
    <link rel="stylesheet" href="/oxidoc.css">
</head>
<body>
    <header class="oxidoc-header">
        <a href="/" class="oxidoc-logo">{project_name}</a>
    </header>
    <div class="oxidoc-layout">
        <aside class="oxidoc-sidebar" role="navigation" aria-label="Documentation navigation">
            {sidebar_html}
        </aside>
        <main class="oxidoc-content" role="main">
            {breadcrumb_html}
            <article>
                {content_html}
            </article>
        </main>
        <aside class="oxidoc-toc-sidebar" role="complementary" aria-label="Table of contents">
            {toc_html}
        </aside>
    </div>
    <script src="/oxidoc-loader.js" type="module" async></script>
</body>
</html>"#,
        lang = config.i18n.default_locale,
    )
}

/// Generate sidebar HTML from resolved navigation groups.
pub fn render_sidebar(groups: &[NavGroup], active_slug: &str) -> String {
    let mut html = String::with_capacity(1024);
    for group in groups {
        if !group.title.is_empty() {
            let _ = write!(
                html,
                r#"<div class="oxidoc-nav-group"><h3 class="oxidoc-nav-title">{}</h3><ul>"#,
                group.title
            );
        } else {
            html.push_str(r#"<div class="oxidoc-nav-group"><ul>"#);
        }
        for page in &group.pages {
            let active = if page.slug == active_slug {
                r#" class="active" aria-current="page""#
            } else {
                ""
            };
            let _ = write!(
                html,
                r#"<li><a href="/{slug}"{active}>{title}</a></li>"#,
                slug = page.slug,
                title = page.title,
            );
        }
        html.push_str("</ul></div>");
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config;
    use crate::crawler::PageEntry;
    use std::path::PathBuf;

    fn test_config() -> OxidocConfig {
        parse_config(r#"[project]\nname = "Test Docs""#.replace(r"\n", "\n").as_str()).unwrap()
    }

    #[test]
    fn render_page_contains_essentials() {
        let config = test_config();
        let html = render_page(&config, "Intro", "<p>Hello</p>", "", "", "", "intro");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Intro — Test Docs</title>"));
        assert!(html.contains("--oxidoc-primary: #3b82f6;"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-loader.js"));
        assert!(html.contains(r#"lang="en""#));
    }

    #[test]
    fn render_sidebar_with_groups() {
        let groups = vec![NavGroup {
            title: "Getting Started".into(),
            pages: vec![
                PageEntry {
                    title: "Intro".into(),
                    slug: "intro".into(),
                    file_path: PathBuf::new(),
                    group: None,
                },
                PageEntry {
                    title: "Setup".into(),
                    slug: "setup".into(),
                    file_path: PathBuf::new(),
                    group: None,
                },
            ],
        }];
        let html = render_sidebar(&groups, "intro");
        assert!(html.contains("Getting Started"));
        assert!(html.contains(r#"aria-current="page""#));
        assert!(html.contains(r#"href="/intro""#));
        assert!(html.contains(r#"href="/setup""#));
    }
}
