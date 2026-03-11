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
    let page_title = if title.is_empty() {
        project_name.clone()
    } else {
        format!("{title} — {project_name}")
    };

    let base_url = config.project.base_url.as_deref().unwrap_or("/");

    let safe_name = crate::utils::html_escape(project_name);
    let logo_html = if let Some(ref logo) = config.project.logo {
        let safe_logo = crate::utils::html_escape(logo);
        format!(
            r#"<a href="/" class="oxidoc-logo"><img src="{safe_logo}" alt="{safe_name}" class="oxidoc-logo-img"> <span>{safe_name}</span></a>"#
        )
    } else {
        format!(r#"<a href="/" class="oxidoc-logo">{safe_name}</a>"#)
    };

    let footer_html = render_footer(config);

    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{page_title}</title>
    <meta name="generator" content="oxidoc">
    <link rel="canonical" href="{base_url}{active_slug}">
    <link rel="preload" href="/oxidoc.css" as="style">
    <link rel="preload" href="/oxidoc-loader.js" as="script">
    <link rel="stylesheet" href="/oxidoc.css">
</head>
<body>
    <a href="#oxidoc-main" class="oxidoc-skip-nav">Skip to content</a>
    <header class="oxidoc-header" role="banner">
        {logo_html}
        <div class="oxidoc-header-actions">
            <button data-oxidoc-search class="oxidoc-search-trigger" aria-label="Search documentation" title="Search (Ctrl+K)">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><circle cx="6.5" cy="6.5" r="5" stroke="currentColor" stroke-width="1.5"/><path d="m10 10 4.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
                <span>Search</span>
            </button>
            <button class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><circle cx="8" cy="8" r="3.5" stroke="currentColor" stroke-width="1.5"/><path d="M8 1v2m0 10v2M1 8h2m10 0h2m-2.05-4.95L11.5 4.5m-7 7L3.05 12.95m9.9 0L11.5 11.5m-7-7L3.05 3.05" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
            </button>
        </div>
    </header>
    <div class="oxidoc-layout">
        <aside class="oxidoc-sidebar" role="navigation" aria-label="Documentation navigation">
            {sidebar_html}
        </aside>
        <main id="oxidoc-main" class="oxidoc-content" role="main">
            {breadcrumb_html}
            <article>
                {content_html}
            </article>
        </main>
        <aside class="oxidoc-toc-sidebar" role="complementary" aria-label="Table of contents">
            {toc_html}
        </aside>
    </div>
    {footer_html}
    <script src="/oxidoc-loader.js" type="module" async></script>
</body>
</html>"##,
        lang = config.i18n.default_locale,
    )
}

/// Render the site footer from config.
fn render_footer(config: &OxidocConfig) -> String {
    let has_copyright = config.footer.copyright.is_some();
    let has_links = !config.footer.links.is_empty();

    if !has_copyright && !has_links {
        return String::new();
    }

    let mut html = String::from(r#"<footer class="oxidoc-footer" role="contentinfo">"#);

    if has_links {
        html.push_str(r#"<nav class="oxidoc-footer-links" aria-label="Footer navigation"><ul>"#);
        for link in &config.footer.links {
            let _ = write!(
                html,
                r#"<li><a href="{}">{}</a></li>"#,
                crate::utils::html_escape(&link.href),
                crate::utils::html_escape(&link.label)
            );
        }
        html.push_str("</ul></nav>");
    }

    if let Some(ref copyright) = config.footer.copyright {
        let _ = write!(
            html,
            r#"<p class="oxidoc-footer-copyright">{}</p>"#,
            crate::utils::html_escape(copyright)
        );
    }

    html.push_str("</footer>");
    html
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
        assert!(html.contains(r#"href="/oxidoc.css""#));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-loader.js"));
        assert!(html.contains(r#"lang="en""#));
    }

    #[test]
    fn render_page_has_skip_nav() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(html.contains("oxidoc-skip-nav"));
        assert!(html.contains(r##"href="#oxidoc-main""##));
        assert!(html.contains(r##"id="oxidoc-main""##));
    }

    #[test]
    fn render_page_has_preload_hints() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(html.contains(r#"rel="preload" href="/oxidoc.css" as="style""#));
        assert!(html.contains(r#"rel="preload" href="/oxidoc-loader.js" as="script""#));
    }

    #[test]
    fn render_page_has_search_and_theme_toggle() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(html.contains("oxidoc-search-trigger"));
        assert!(html.contains("oxidoc-theme-toggle"));
    }

    #[test]
    fn render_page_with_logo() {
        let config = parse_config("[project]\nname = \"Test\"\nlogo = \"/logo.svg\"").unwrap();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(html.contains(r#"src="/logo.svg""#));
        assert!(html.contains("oxidoc-logo-img"));
    }

    #[test]
    fn render_page_with_footer() {
        let config = parse_config(
            "[project]\nname = \"T\"\n[footer]\ncopyright = \"2024 Acme\"\n[[footer.links]]\nlabel = \"GitHub\"\nhref = \"https://github.com\"",
        )
        .unwrap();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(html.contains("2024 Acme"));
        assert!(html.contains("GitHub"));
        assert!(html.contains("oxidoc-footer"));
    }

    #[test]
    fn render_page_no_footer_if_empty() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "");
        assert!(!html.contains("oxidoc-footer"));
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
