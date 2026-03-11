use crate::config::OxidocConfig;
use crate::crawler::NavGroup;
use std::fmt::Write;

const HEADER_ACTIONS_HTML: &str = r#"<div class="oxidoc-header-actions">
            <button data-oxidoc-search class="oxidoc-search-trigger" aria-label="Search documentation" title="Search (Ctrl+K)">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><circle cx="6.5" cy="6.5" r="5" stroke="currentColor" stroke-width="1.5"/><path d="m10 10 4.5 4.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
                <span>Search</span>
            </button>
            <button class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden="true"><circle cx="8" cy="8" r="3.5" stroke="currentColor" stroke-width="1.5"/><path d="M8 1v2m0 10v2M1 8h2m10 0h2m-2.05-4.95L11.5 4.5m-7 7L3.05 12.95m9.9 0L11.5 11.5m-7-7L3.05 3.05" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
            </button>
        </div>"#;

/// Generate the logo HTML for the header.
fn render_logo_html(config: &OxidocConfig) -> (String, String) {
    let safe_name = crate::utils::html_escape(&config.project.name);
    let logo_html = if let Some(ref logo) = config.project.logo {
        let safe_logo = crate::utils::html_escape(logo);
        format!(
            r#"<a href="/" class="oxidoc-logo"><img src="{safe_logo}" alt="{safe_name}" class="oxidoc-logo-img"> <span>{safe_name}</span></a>"#
        )
    } else {
        format!(r#"<a href="/" class="oxidoc-logo">{safe_name}</a>"#)
    };
    (logo_html, safe_name)
}

/// Wrap rendered page content in a full HTML document.
#[allow(clippy::too_many_arguments)]
pub fn render_page(
    config: &OxidocConfig,
    title: &str,
    content_html: &str,
    toc_html: &str,
    sidebar_html: &str,
    breadcrumb_html: &str,
    active_slug: &str,
    description: Option<&str>,
) -> String {
    let project_name = &config.project.name;
    let page_title = if title.is_empty() {
        project_name.clone()
    } else {
        format!("{title} - {project_name}")
    };

    let base_url = config.project.base_url.as_deref().unwrap_or("/");
    let (logo_html, safe_name) = render_logo_html(config);

    let footer_html = render_footer(config);

    // Determine page description for SEO
    let default_description = format!("{} documentation", project_name);
    let page_description = description
        .or(config.project.description.as_deref())
        .unwrap_or(default_description.as_str());

    // Build JSON-LD structured data
    let safe_url = if base_url.ends_with('/') {
        format!("{}{}.html", base_url, active_slug)
    } else {
        format!("{}/{}.html", base_url, active_slug)
    };

    let json_ld = format!(
        r##"{{"@context":"https://schema.org","@type":"WebPage","name":{},"description":{},"url":{},"site":{{name":{}}}}}"##,
        serde_json::to_string(&page_title).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&page_description).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&safe_url).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&project_name).unwrap_or_else(|_| "null".into()),
    );

    let page_description_escaped = crate::utils::html_escape(page_description);
    let page_title_escaped = crate::utils::html_escape(&page_title);

    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{page_title_escaped}</title>
    <meta name="description" content="{page_description_escaped}">
    <meta name="generator" content="oxidoc">
    <meta property="og:title" content="{page_title_escaped}">
    <meta property="og:type" content="article">
    <meta property="og:url" content="{safe_url}">
    <meta property="og:site_name" content="{safe_name}">
    <meta property="og:description" content="{page_description_escaped}">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{page_title_escaped}">
    <script type="application/ld+json">{json_ld}</script>
    <link rel="canonical" href="{base_url}{active_slug}">
    <link rel="preload" href="/oxidoc.css" as="style">
    <link rel="preload" href="/oxidoc-loader.js" as="script">
    <link rel="stylesheet" href="/oxidoc.css">
</head>
<body>
    <a href="#oxidoc-main" class="oxidoc-skip-nav">Skip to content</a>
    <header class="oxidoc-header" role="banner">
        {logo_html}
        {HEADER_ACTIONS_HTML}
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
        page_description_escaped = page_description_escaped,
        page_title_escaped = page_title_escaped,
        safe_url = safe_url,
        safe_name = safe_name,
        json_ld = json_ld,
        base_url = base_url,
        active_slug = active_slug,
        logo_html = logo_html,
        sidebar_html = sidebar_html,
        breadcrumb_html = breadcrumb_html,
        content_html = content_html,
        toc_html = toc_html,
        footer_html = footer_html,
        HEADER_ACTIONS_HTML = HEADER_ACTIONS_HTML,
    )
}

/// Generate a 404 error page using the site template.
pub fn render_404_page(config: &OxidocConfig) -> String {
    let (logo_html, safe_name) = render_logo_html(config);

    let footer_html = render_footer(config);

    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Page Not Found - {project_name}</title>
    <meta name="description" content="The page you are looking for could not be found.">
    <meta name="generator" content="oxidoc">
    <link rel="preload" href="/oxidoc.css" as="style">
    <link rel="preload" href="/oxidoc-loader.js" as="script">
    <link rel="stylesheet" href="/oxidoc.css">
</head>
<body>
    <a href="#oxidoc-main" class="oxidoc-skip-nav">Skip to content</a>
    <header class="oxidoc-header" role="banner">
        {logo_html}
        {HEADER_ACTIONS_HTML}
    </header>
    <div class="oxidoc-layout">
        <main id="oxidoc-main" class="oxidoc-content" role="main">
            <article>
                <h1>404 - Page Not Found</h1>
                <p>The page you are looking for could not be found. Please check the URL or use the search function above.</p>
                <p><a href="/">Return to home</a></p>
            </article>
        </main>
    </div>
    {footer_html}
    <script src="/oxidoc-loader.js" type="module" async></script>
</body>
</html>"##,
        lang = config.i18n.default_locale,
        project_name = safe_name,
        logo_html = logo_html,
        footer_html = footer_html,
        HEADER_ACTIONS_HTML = HEADER_ACTIONS_HTML,
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
        parse_config(
            r#"[project]
name = "Test Docs""#,
        )
        .unwrap()
    }

    #[test]
    fn render_page_contains_essentials() {
        let config = test_config();
        let html = render_page(&config, "Intro", "<p>Hello</p>", "", "", "", "intro", None);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Intro - Test Docs</title>"));
        assert!(html.contains(r#"href="/oxidoc.css""#));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-loader.js"));
        assert!(html.contains(r#"lang="en""#));
    }

    #[test]
    fn render_page_has_accessibility_and_preload() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "", None);
        assert!(html.contains("oxidoc-skip-nav"));
        assert!(html.contains(r##"href="#oxidoc-main""##));
        assert!(html.contains(r##"id="oxidoc-main""##));
        assert!(html.contains(r#"rel="preload" href="/oxidoc.css" as="style""#));
        assert!(html.contains(r#"rel="preload" href="/oxidoc-loader.js" as="script""#));
        assert!(html.contains("oxidoc-search-trigger"));
        assert!(html.contains("oxidoc-theme-toggle"));
    }

    #[test]
    fn render_page_with_logo() {
        let config = parse_config(
            r#"[project]
name = "Test"
logo = "/logo.svg""#,
        )
        .unwrap();
        let html = render_page(&config, "", "", "", "", "", "", None);
        assert!(html.contains(r#"src="/logo.svg""#));
        assert!(html.contains("oxidoc-logo-img"));
    }

    #[test]
    fn render_page_with_footer() {
        let config = parse_config(
            r#"[project]
name = "T"

[footer]
copyright = "2024 Acme"

[[footer.links]]
label = "GitHub"
href = "https://github.com""#,
        )
        .unwrap();
        let html = render_page(&config, "", "", "", "", "", "", None);
        assert!(html.contains("2024 Acme"));
        assert!(html.contains("GitHub"));
        assert!(html.contains("oxidoc-footer"));
    }

    #[test]
    fn render_page_no_footer_if_empty() {
        let config = test_config();
        let html = render_page(&config, "", "", "", "", "", "", None);
        assert!(!html.contains("oxidoc-footer"));
    }

    #[test]
    fn render_page_includes_seo_meta_tags() {
        let config = test_config();
        let html = render_page(
            &config,
            "Test Page",
            "",
            "",
            "",
            "",
            "test",
            Some("A test page"),
        );
        assert!(html.contains(r#"<meta name="description" content="A test page">"#));
        assert!(html.contains(r#"<meta property="og:title""#));
        assert!(html.contains(r#"<meta property="og:type" content="article""#));
        assert!(html.contains(r#"<meta property="og:description""#));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary""#));
        assert!(html.contains("application/ld+json"));
    }

    #[test]
    fn render_page_uses_config_description_fallback() {
        let config =
            parse_config("[project]\nname = \"Test\"\ndescription = \"Default description\"")
                .unwrap();
        let html = render_page(&config, "Page", "", "", "", "", "page", None);
        assert!(html.contains("Default description"));
    }

    #[test]
    fn render_404_page_contains_essentials() {
        let config = test_config();
        let html = render_404_page(&config);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("Return to home"));
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
