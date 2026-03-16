use crate::config::OxidocConfig;
use crate::i18n::I18nState;
use crate::search_provider::SearchProvider;
use crate::template_assets::{
    AssetConfig, build_preload_links, build_script_tag, build_stylesheet_link,
};
use crate::template_parts::{render_analytics_script, render_footer};

pub(crate) const SCROLLSPY_JS: &str = include_str!("templates/scrollspy.js");
pub(crate) const HEADER_SCROLL_JS: &str = include_str!("templates/header_scroll.js");
pub(crate) const BACK_TO_TOP_JS: &str = include_str!("templates/back_to_top.js");
pub(crate) const THEME_TOGGLE_JS: &str = include_str!("templates/theme_toggle.js");
pub(crate) const SEARCH_DIALOG_JS: &str = include_str!("templates/search_dialog.js");
pub(crate) const API_TABS_JS: &str = include_str!("templates/api_tabs.js");
pub(crate) const SEARCH_DIALOG_HTML: &str = include_str!("templates/search_dialog.html");
pub(crate) const HEADER_ACTIONS_HTML: &str = include_str!("templates/header_actions.html");
pub(crate) const MOBILE_MENU_JS: &str = include_str!("templates/mobile_menu.js");
pub(crate) const ERROR_404_HTML: &str = include_str!("templates/error_404.html");

use crate::config::SocialConfig;

/// Build the header navigation links HTML.
pub(crate) fn build_header_nav(links: &[crate::config::HeaderLink]) -> String {
    if links.is_empty() {
        return String::new();
    }
    let mut html = String::from(r#"<nav class="oxidoc-header-nav">"#);
    for link in links {
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<a href="{}" class="oxidoc-header-nav-link">{}</a>"#,
            crate::utils::html_escape(&link.href),
            crate::utils::html_escape(&link.label),
        );
    }
    html.push_str("</nav>");
    html
}

/// Build the mobile menu links (header links rendered as a list for mobile sidebar).
pub(crate) fn build_mobile_nav_links(links: &[crate::config::HeaderLink]) -> String {
    if links.is_empty() {
        return String::new();
    }
    let mut html =
        String::from(r#"<nav class="oxidoc-mobile-nav-links" aria-label="Main navigation"><ul>"#);
    for link in links {
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<li><a href="{}">{}</a></li>"#,
            crate::utils::html_escape(&link.href),
            crate::utils::html_escape(&link.label),
        );
    }
    html.push_str("</ul></nav>");
    html
}

/// Build the header actions HTML with social links and icons injected.
pub(crate) fn build_header_actions(social: &SocialConfig) -> String {
    use crate::icons;

    let social_html = social.render_header_icons();
    let theme_icon = icons::svg_icon(icons::CONTRAST, "20", "20", "");
    let search_icon = icons::svg_icon(icons::SEARCH, "16", "16", "");

    HEADER_ACTIONS_HTML
        .replacen(
            r#"<div class="oxidoc-header-actions">"#,
            &format!(r#"<div class="oxidoc-header-actions">{social_html}"#),
            1,
        )
        .replacen(
            r#"class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme"></button>"#,
            &format!(r#"class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme">{theme_icon}</button>"#),
            1,
        )
        .replacen(
            r#"<span>Search</span>"#,
            &format!(r#"{search_icon}<span>Search</span>"#),
            1,
        )
}

/// Build the mobile menu toggle button (placed before logo in header).
pub(crate) fn build_menu_toggle() -> String {
    use crate::icons;

    let menu_icon = icons::svg_icon(icons::MENU, "24", "24", "oxidoc-menu-icon");
    let close_icon = icons::svg_icon(icons::CLOSE, "24", "24", "oxidoc-close-icon");

    format!(
        r#"<button class="oxidoc-menu-toggle" aria-label="Open navigation menu" aria-expanded="false">{menu_icon}{close_icon}</button>"#
    )
}

/// Generate the logo HTML for the header.
pub(crate) fn render_logo_html(config: &OxidocConfig) -> (String, String) {
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
    page_meta_html: &str,
    assets: &AssetConfig<'_>,
    locale: &str,
    i18n_state: &I18nState,
    search_provider: &SearchProvider,
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
    let safe_url = if active_slug.is_empty() {
        if base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{}/", base_url)
        }
    } else if base_url.ends_with('/') {
        format!("{}{}", base_url, active_slug)
    } else {
        format!("{}/{}", base_url, active_slug)
    };

    let json_ld = format!(
        r##"{{"@context":"https://schema.org","@type":"WebPage","name":{},"description":{},"url":{},"site":{{"name":{}}}}}"##,
        serde_json::to_string(&page_title).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&page_description).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&safe_url).unwrap_or_else(|_| "null".into()),
        serde_json::to_string(&project_name).unwrap_or_else(|_| "null".into()),
    );

    let page_description_escaped = crate::utils::html_escape(page_description);
    let page_title_escaped = crate::utils::html_escape(&page_title);

    let css_href = assets.css_path.unwrap_or("/oxidoc.css");
    let js_src = assets.js_path.unwrap_or("/oxidoc-loader.js");
    let analytics_html = render_analytics_script(config);
    let stylesheet_link = build_stylesheet_link(css_href, assets.css_sri);
    let script_tag = build_script_tag(js_src, assets.js_sri);
    let (css_preload, js_preload) =
        build_preload_links(css_href, assets.css_sri, js_src, assets.js_sri);

    // Generate search provider head tags and scripts
    let search_head_tags = search_provider.render_head_tags();
    let search_scripts = search_provider.render_scripts();

    // Generate locale switcher if i18n is enabled
    let current_path = format!("/{}", active_slug);
    let locale_switcher_html = i18n_state.render_locale_switcher(locale, &current_path);

    let favicon_html = if let Some(ref favicon) = config.project.favicon {
        let favicon_escaped = crate::utils::html_escape(favicon);
        if favicon.ends_with(".svg") {
            format!(r#"    <link rel="icon" type="image/svg+xml" href="{favicon_escaped}">"#)
        } else if favicon.ends_with(".png") {
            format!(r#"    <link rel="icon" type="image/png" href="{favicon_escaped}">"#)
        } else {
            format!(r#"    <link rel="icon" href="{favicon_escaped}">"#)
        }
    } else {
        String::new()
    };

    // Mobile TOC dropdown (shown only on small screens via CSS)
    let mobile_toc_html = if toc_html.is_empty() {
        String::new()
    } else {
        // Extract the <ul>...</ul> from the toc_html nav
        let toc_list = toc_html
            .find("<ul>")
            .and_then(|start| toc_html.rfind("</ul>").map(|end| &toc_html[start..end + 5]))
            .unwrap_or(toc_html);
        format!(
            r#"<div class="oxidoc-toc-mobile"><button class="oxidoc-toc-mobile-toggle" aria-expanded="false"><span>On this page</span><svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"><path fill="currentColor" d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6l-6-6z"/></svg></button><nav class="oxidoc-toc-mobile-dropdown" aria-label="Table of contents">{toc_list}</nav></div>"#
        )
    };

    let toc_aside = if toc_html.is_empty() {
        r#"<aside class="oxidoc-toc-sidebar" role="complementary" aria-label="Table of contents"></aside>"#.to_string()
    } else {
        format!(
            r#"<aside class="oxidoc-toc-sidebar" role="complementary" aria-label="Table of contents"><div class="oxidoc-toc-inner">{toc_html}</div></aside>"#
        )
    };

    let html = format!(
        r##"<!DOCTYPE html>
<html lang="{lang}" data-locale="{locale}">
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
{favicon_html}
{css_preload}
{js_preload}
{stylesheet_link}
    {analytics_html}
    {search_head_tags}
</head>
<body data-locale="{locale}">
<a href="#oxidoc-main" class="oxidoc-skip-nav">Skip to content</a>
    <header class="oxidoc-header" role="banner">
        {menu_toggle_html}
        {logo_html}
        {header_nav_html}
        {locale_switcher_html}
        {header_actions_html}
    </header>
    <div class="oxidoc-sidebar-overlay"></div>
    <div class="oxidoc-layout">
        <aside class="oxidoc-sidebar" role="navigation" aria-label="Documentation navigation">
            <div class="oxidoc-sidebar-inner">
                {mobile_nav_links}
                <button class="oxidoc-mobile-back-btn">&#8592; Back to main menu</button>
                <div class="oxidoc-sidebar-doc-nav">
                    {sidebar_html}
                </div>
            </div>
        </aside>
        <main id="oxidoc-main" class="oxidoc-content" role="main">
            {mobile_toc_html}
            {breadcrumb_html}
            <article>
                {content_html}
            </article>
            {page_meta_html}
        </main>
        {toc_aside}
    </div>
    {footer_html}
{script_tag}
    {search_scripts}

</body>
</html>"##,
        lang = locale,
        locale = locale,
        page_description_escaped = page_description_escaped,
        page_title_escaped = page_title_escaped,
        safe_url = safe_url,
        safe_name = safe_name,
        json_ld = json_ld,
        base_url = base_url,
        active_slug = active_slug,
        logo_html = logo_html,
        locale_switcher_html = locale_switcher_html,
        sidebar_html = sidebar_html,
        breadcrumb_html = breadcrumb_html,
        content_html = content_html,
        toc_aside = toc_aside,
        page_meta_html = page_meta_html,
        footer_html = footer_html,
        css_preload = css_preload,
        js_preload = js_preload,
        analytics_html = analytics_html,
        stylesheet_link = stylesheet_link,
        script_tag = script_tag,
        search_head_tags = search_head_tags,
        search_scripts = search_scripts,
        header_nav_html = build_header_nav(&config.routing.header_links),
        menu_toggle_html = build_menu_toggle(),
        header_actions_html = build_header_actions(&config.social),
        mobile_nav_links = build_mobile_nav_links(&config.routing.header_links),
        mobile_toc_html = mobile_toc_html,
    );
    let mut html = html;
    // Extract <Head> component content and move to <head>
    let mut extra_head = String::new();
    while let Some(start) = html.find("<!--oxidoc-head-start-->") {
        let end_marker = "<!--oxidoc-head-end-->";
        if let Some(end) = html[start..].find(end_marker) {
            let content_start = start + "<!--oxidoc-head-start-->".len();
            let content_end = start + end;
            extra_head.push_str(&html[content_start..content_end]);
            html.replace_range(start..content_end + end_marker.len(), "");
        } else {
            break;
        }
    }
    if !extra_head.is_empty() {
        html = html.replace("</head>", &format!("{extra_head}\n</head>"));
    }

    // Conditionally inject mermaid.js if the page has mermaid diagrams
    let mermaid_script = if html.contains(r#"<pre class="mermaid">"#) {
        concat!(
            "<script type=\"module\">",
            include_str!("templates/mermaid_init.js"),
            "</script>"
        )
    } else {
        ""
    };

    // Inject search dialog + scripts (contain curly braces, can't go in format!)
    let search_html = if config.search.semantic {
        SEARCH_DIALOG_HTML.replacen(
            r#"class="oxidoc-search-dialog""#,
            r#"class="oxidoc-search-dialog" data-semantic="true""#,
            1,
        )
    } else {
        SEARCH_DIALOG_HTML.to_string()
    };
    html.replace(
        "</body>",
        &format!(
            "{}\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n{}</body>",
            search_html, THEME_TOGGLE_JS, SEARCH_DIALOG_JS, SCROLLSPY_JS, HEADER_SCROLL_JS, BACK_TO_TOP_JS, API_TABS_JS, MOBILE_MENU_JS, mermaid_script
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config;
    use crate::template_assets::AssetConfig;

    fn test_config() -> OxidocConfig {
        parse_config(
            r#"[project]
name = "Test Docs""#,
        )
        .unwrap()
    }

    fn default_assets() -> AssetConfig<'static> {
        AssetConfig::default()
    }

    fn default_i18n_state() -> crate::i18n::I18nState {
        crate::i18n::I18nState::from_config("en", &[])
    }

    fn default_search_provider() -> SearchProvider {
        SearchProvider::Oxidoc { model_path: None }
    }

    /// Render a page with test defaults; only title, content, slug, and description vary.
    fn render_test_page(
        config: &OxidocConfig,
        title: &str,
        content: &str,
        slug: &str,
        desc: Option<&str>,
    ) -> String {
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        render_page(
            config,
            title,
            content,
            "",
            "",
            "",
            slug,
            desc,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
        )
    }

    /// Like `render_test_page` but with custom assets.
    fn render_test_page_with_assets(
        config: &OxidocConfig,
        title: &str,
        slug: &str,
        assets: &AssetConfig<'_>,
    ) -> String {
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        render_page(
            config, title, "", "", "", "", slug, None, "", assets, "en", &i18n, &provider,
        )
    }

    #[test]
    fn render_page_structure_and_accessibility() {
        let config = test_config();
        let html = render_test_page(&config, "Intro", "<p>Hello</p>", "intro", None);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Intro - Test Docs</title>"));
        assert!(html.contains(r#"href="/oxidoc.css""#));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-loader.js"));
        assert!(html.contains(r#"lang="en""#));
        assert!(html.contains("oxidoc-skip-nav"));
        assert!(html.contains(r##"href="#oxidoc-main""##));
        assert!(html.contains(r##"id="oxidoc-main""##));
        assert!(html.contains(r#"rel="preload" href="/oxidoc.css" as="style""#));
        assert!(html.contains(r#"rel="preload" href="/oxidoc-loader.js" as="script""#));
        assert!(html.contains("oxidoc-search-trigger"));
        assert!(html.contains("oxidoc-theme-toggle"));
        assert!(html.contains("oxidoc-footer"));

        let logo_cfg = parse_config("[project]\nname = \"T\"\nlogo = \"/logo.svg\"").unwrap();
        let html = render_test_page(&logo_cfg, "", "", "", None);
        assert!(html.contains(r#"src="/logo.svg""#) && html.contains("oxidoc-logo-img"));
    }

    #[test]
    fn render_page_seo_and_description() {
        let html = render_test_page(&test_config(), "Test Page", "", "test", Some("A test page"));
        assert!(html.contains(r#"<meta name="description" content="A test page">"#));
        assert!(html.contains(r#"<meta property="og:title""#));
        assert!(html.contains(r#"<meta property="og:type" content="article""#));
        assert!(html.contains(r#"<meta property="og:description""#));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary""#));
        assert!(html.contains("application/ld+json"));

        let cfg = parse_config("[project]\nname = \"T\"\ndescription = \"Fallback desc\"").unwrap();
        let html = render_test_page(&cfg, "P", "", "p", None);
        assert!(html.contains("Fallback desc"));
    }

    #[test]
    fn render_page_with_custom_assets_and_sri() {
        let config = test_config();
        let assets = AssetConfig {
            css_path: Some("/oxidoc.a1b2c3d4.css"),
            js_path: Some("/oxidoc-loader.h5i6j7k8.js"),
            ..Default::default()
        };
        let html = render_test_page_with_assets(&config, "Test", "test", &assets);
        assert!(html.contains(r#"href="/oxidoc.a1b2c3d4.css""#));
        assert!(html.contains(r#"src="/oxidoc-loader.h5i6j7k8.js""#));

        let sri_assets = AssetConfig {
            css_sri: Some("sha384-abc123"),
            js_sri: Some("sha384-def456"),
            ..assets
        };
        let html = render_test_page_with_assets(&config, "Test", "test", &sri_assets);
        assert!(html.contains(r#"integrity="sha384-abc123""#));
        assert!(html.contains(r#"integrity="sha384-def456""#));
        assert!(html.contains(r#"crossorigin="anonymous""#));
    }

    #[test]
    fn render_page_includes_analytics() {
        let ga_config = parse_config(
            "[project]\nname = \"Test\"\n\n[analytics]\ngoogle_analytics = \"G-XXXXXXXXXX\"",
        )
        .unwrap();
        let html = render_test_page(&ga_config, "", "", "", None);
        assert!(html.contains("googletagmanager.com") && html.contains("G-XXXXXXXXXX"));

        let custom_config = parse_config(
            "[project]\nname = \"Test\"\n\n[analytics]\nscript = \"<script>custom</script>\"",
        )
        .unwrap();
        let html = render_test_page(&custom_config, "", "", "", None);
        assert!(html.contains("custom"));
    }
}
