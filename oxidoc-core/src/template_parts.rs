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
pub fn render_footer(config: &OxidocConfig) -> String {
    let has_copyright = config.footer.copyright_owner.is_some();
    let has_links = !config.footer.links.is_empty();
    let has_attribution = config.attribution.oxidoc;

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

    // Combined copyright + attribution line (Docusaurus-style)
    // e.g., "Copyright © 2026 Oxidoc. Built with Oxidoc."
    if has_copyright || has_attribution {
        html.push_str(r#"<p class="oxidoc-footer-copyright">"#);
        let mut parts = Vec::new();

        if let Some(ref owner) = config.footer.copyright_owner {
            let year = chrono::Utc::now().format("%Y");
            let owner_html = if let Some(ref url) = config.footer.copyright_owner_url {
                format!(
                    r#"<a href="{}">{}</a>"#,
                    crate::utils::html_escape(url),
                    crate::utils::html_escape(owner)
                )
            } else {
                crate::utils::html_escape(owner)
            };
            parts.push(format!("Copyright \u{00a9} {year} {owner_html}."));
        }

        if config.attribution.oxidoc {
            parts.push(r#"Built with <a href="https://oxidoc.dev">Oxidoc</a>."#.to_string());
        }

        let _ = write!(html, "{}", parts.join(" "));
        html.push_str("</p>");
    }

    html.push_str("</footer>");
    html
}

/// Info about adjacent pages for prev/next navigation.
#[derive(Debug, Default)]
pub struct PageNav {
    pub prev: Option<(String, String)>, // (slug, title)
    pub next: Option<(String, String)>,
}

/// Git metadata for a page.
#[derive(Debug, Default)]
pub struct PageGitMeta {
    pub last_updated: Option<String>, // formatted date like "Mar 6, 2026"
    pub last_author: Option<String>,
}

/// Render the page-bottom meta bar: "Edit this page" link, last-updated, and prev/next nav.
pub fn render_page_meta(
    config: &OxidocConfig,
    slug: &str,
    nav: &PageNav,
    git_meta: &PageGitMeta,
    homepage_slug: Option<&str>,
) -> String {
    let has_edit = config.project.edit_url.is_some();
    let has_git = git_meta.last_updated.is_some();
    let has_nav = nav.prev.is_some() || nav.next.is_some();

    if !has_edit && !has_git && !has_nav {
        return String::new();
    }

    let mut html = String::from(r#"<div class="oxidoc-page-meta">"#);

    // Edit link + last updated row
    if has_edit || has_git {
        html.push_str(r#"<div class="oxidoc-page-meta-row">"#);
        if let Some(ref base) = config.project.edit_url {
            let view_href = format!("{}/{}.rdx?plain=1", base.trim_end_matches('/'), slug);
            let _ = write!(
                html,
                r#"<a href="{}" class="oxidoc-edit-link" target="_blank" rel="noopener">{}</a>"#,
                crate::utils::html_escape(&view_href),
                crate::utils::html_escape(&config.project.edit_label)
            );
        } else {
            html.push_str("<span></span>");
        }
        if let Some(ref date) = git_meta.last_updated {
            html.push_str(r#"<span class="oxidoc-last-updated">Last updated on <strong>"#);
            html.push_str(&crate::utils::html_escape(date));
            html.push_str("</strong>");
            if let Some(ref author) = git_meta.last_author {
                html.push_str(" by ");
                html.push_str(&crate::utils::html_escape(author));
            }
            html.push_str("</span>");
        }
        html.push_str("</div>");
    }

    // Prev/Next navigation
    if has_nav {
        html.push_str(r#"<nav class="oxidoc-page-nav" aria-label="Page navigation">"#);
        if let Some((ref slug, ref title)) = nav.prev {
            let href = if homepage_slug == Some(slug.as_str()) {
                "/".to_string()
            } else {
                format!("/{}", crate::utils::html_escape(slug))
            };
            let _ = write!(
                html,
                r#"<a href="{href}" class="oxidoc-page-nav-prev"><span class="oxidoc-page-nav-label">Previous</span><span class="oxidoc-page-nav-title">&laquo; {}</span></a>"#,
                crate::utils::html_escape(title)
            );
        } else {
            html.push_str(r#"<span></span>"#);
        }
        if let Some((ref slug, ref title)) = nav.next {
            let href = if homepage_slug == Some(slug.as_str()) {
                "/".to_string()
            } else {
                format!("/{}", crate::utils::html_escape(slug))
            };
            let _ = write!(
                html,
                r#"<a href="{href}" class="oxidoc-page-nav-next"><span class="oxidoc-page-nav-label">Next</span><span class="oxidoc-page-nav-title">{} &raquo;</span></a>"#,
                crate::utils::html_escape(title)
            );
        } else {
            html.push_str(r#"<span></span>"#);
        }
        html.push_str("</nav>");
    }

    html.push_str("</div>");
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
        if group.pages.is_empty() {
            continue;
        }
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
            } else if page.slug.ends_with("/index") {
                // lib/index → /lib
                let parent = &page.slug[..page.slug.len() - "/index".len()];
                format!("/{}", crate::utils::html_escape(parent))
            } else {
                format!("/{}", crate::utils::html_escape(&page.slug))
            };
            let _ = write!(
                html,
                r#"<li><a href="{href}"{active}>{title}</a></li>"#,
                title = crate::utils::html_escape(&page.short_title),
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

    #[test]
    fn render_sidebar_with_groups() {
        let groups = vec![NavGroup {
            title: "Getting Started".into(),
            pages: vec![
                PageEntry {
                    title: "Intro".into(),
                    short_title: "Intro".into(),
                    slug: "intro".into(),
                    file_path: PathBuf::new(),
                    group: None,
                },
                PageEntry {
                    title: "Setup".into(),
                    short_title: "Setup".into(),
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
copyright_owner = "Acme"

[[footer.links]]
label = "GitHub"
href = "https://github.com""#,
        )
        .unwrap();
        let html = render_footer(&config);
        assert!(html.contains("Acme"));
        assert!(html.contains("Copyright ©"));
        assert!(html.contains("GitHub"));
        assert!(html.contains("oxidoc-footer"));
    }

    #[test]
    fn render_footer_with_default_attribution() {
        let config = parse_config("[project]\nname = \"T\"").unwrap();
        let html = render_footer(&config);
        // Default config has attribution.oxidoc = true, so footer is present
        assert!(html.contains("oxidoc-footer"));
        assert!(html.contains("Built with"));
    }

    #[test]
    fn render_footer_empty_when_attribution_disabled() {
        let config =
            parse_config("[project]\nname = \"T\"\n\n[attribution]\noxidoc = false").unwrap();
        let html = render_footer(&config);
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
