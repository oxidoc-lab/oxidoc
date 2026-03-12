use crate::config::OxidocConfig;
use crate::crawler::NavGroup;
use std::fmt::Write;

/// Render analytics scripts if configured.
pub fn render_analytics_script(config: &OxidocConfig) -> String {
    let mut html = String::new();

    // Custom analytics script (e.g., GTM, Plausible)
    if let Some(ref script) = config.analytics.script {
        html.push_str(script);
        html.push('\n');
    }

    // Google Analytics
    if let Some(ref ga_id) = config.analytics.google_analytics {
        let safe_id = crate::utils::html_escape(ga_id);
        html.push_str(&format!(
            r##"<script async src="https://www.googletagmanager.com/gtag/js?id={}"></script>
<script>
window.dataLayer = window.dataLayer || [];
function gtag(){{dataLayer.push(arguments);}}
gtag('js', new Date());
gtag('config', '{}');
</script>"##,
            safe_id, safe_id
        ));
    }

    html
}

/// Render the site footer from config, with optional attribution.
pub fn render_footer(config: &OxidocConfig, theme: &crate::theme::ResolvedTheme) -> String {
    let has_copyright = config.footer.copyright.is_some();
    let has_links = !config.footer.links.is_empty();
    let attribution = &config.attribution;
    let has_attribution = attribution.oxidoc || attribution.theme;

    if !has_copyright && !has_links && !has_attribution {
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

    // Attribution line
    if has_attribution {
        html.push_str(r#"<p class="oxidoc-footer-attribution">"#);
        let mut parts = Vec::new();

        if attribution.oxidoc {
            parts.push(
                r#"Built with <a href="https://github.com/oxidoc/oxidoc">Oxidoc</a>"#.to_string(),
            );
        }

        if attribution.theme && theme.name != "oxidoc" {
            let theme_name = crate::utils::html_escape(&theme.name);
            let theme_part = match (&theme.url, &theme.author) {
                (Some(url), Some(author)) => {
                    let safe_url = crate::utils::html_escape(url);
                    let safe_author = crate::utils::html_escape(author);
                    format!(r#"Theme: <a href="{safe_url}">{theme_name}</a> by {safe_author}"#)
                }
                (Some(url), None) => {
                    let safe_url = crate::utils::html_escape(url);
                    format!(r#"Theme: <a href="{safe_url}">{theme_name}</a>"#)
                }
                (None, Some(author)) => {
                    let safe_author = crate::utils::html_escape(author);
                    format!(r#"Theme: {theme_name} by {safe_author}"#)
                }
                (None, None) => {
                    format!(r#"Theme: {theme_name}"#)
                }
            };
            parts.push(theme_part);
        }

        let _ = write!(html, "{}", parts.join(" · "));
        html.push_str("</p>");
    }

    html.push_str("</footer>");
    html
}

/// Generate sidebar HTML from resolved navigation groups.
pub fn render_sidebar(groups: &[NavGroup], active_slug: &str) -> String {
    render_sidebar_with_homepage(groups, active_slug, None)
}

pub fn render_sidebar_with_homepage(
    groups: &[NavGroup],
    active_slug: &str,
    homepage_slug: Option<&str>,
) -> String {
    let mut html = String::with_capacity(1024);
    for group in groups {
        if !group.title.is_empty() {
            let _ = write!(
                html,
                r#"<div class="oxidoc-nav-group"><h3 class="oxidoc-nav-title">{}</h3><ul>"#,
                crate::utils::html_escape(&group.title)
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
            let href = if homepage_slug == Some(page.slug.as_str()) {
                "/".to_string()
            } else {
                format!("/{}", crate::utils::html_escape(&page.slug))
            };
            let _ = write!(
                html,
                r#"<li><a href="{href}"{active}>{title}</a></li>"#,
                title = crate::utils::html_escape(&page.title),
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

    fn test_theme() -> crate::theme::ResolvedTheme {
        crate::theme::builtin_theme("oxidoc").unwrap()
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

    #[test]
    fn render_footer_with_links_and_copyright() {
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
        let html = render_footer(&config, &test_theme());
        assert!(html.contains("2024 Acme"));
        assert!(html.contains("GitHub"));
        assert!(html.contains("oxidoc-footer"));
    }

    #[test]
    fn render_footer_with_default_attribution() {
        let config = parse_config("[project]\nname = \"T\"").unwrap();
        let html = render_footer(&config, &test_theme());
        // Default config has attribution.oxidoc = true, so footer is present
        assert!(html.contains("oxidoc-footer"));
        assert!(html.contains("Built with"));
    }

    #[test]
    fn render_footer_empty_when_attribution_disabled() {
        let config =
            parse_config("[project]\nname = \"T\"\n\n[attribution]\noxidoc = false\ntheme = false")
                .unwrap();
        let html = render_footer(&config, &test_theme());
        assert!(html.is_empty());
    }

    #[test]
    fn render_analytics_google() {
        let config = parse_config(
            "[project]\nname = \"T\"\n\n[analytics]\ngoogle_analytics = \"G-TEST123\"",
        )
        .unwrap();
        let html = render_analytics_script(&config);
        assert!(html.contains("googletagmanager.com"));
        assert!(html.contains("G-TEST123"));
    }
}
