use crate::config::OxidocConfig;
use crate::i18n::I18nState;
use crate::search_provider::SearchProvider;
use crate::template_assets::{
    AssetConfig, build_preload_links, build_script_tag, build_stylesheet_link,
};
use crate::template_parts::{render_analytics_script, render_footer};
use crate::theme::ResolvedTheme;

pub(crate) const SCROLLSPY_JS: &str = include_str!("templates/scrollspy.js");
pub(crate) const HEADER_SCROLL_JS: &str = include_str!("templates/header_scroll.js");
pub(crate) const BACK_TO_TOP_JS: &str = include_str!("templates/back_to_top.js");
pub(crate) const THEME_TOGGLE_JS: &str = include_str!("templates/theme_toggle.js");
pub(crate) const SEARCH_DIALOG_JS: &str = include_str!("templates/search_dialog.js");
pub(crate) const SEARCH_DIALOG_HTML: &str = include_str!("templates/search_dialog.html");
pub(crate) const HEADER_ACTIONS_HTML: &str = include_str!("templates/header_actions.html");
pub(crate) const ERROR_404_HTML: &str = include_str!("templates/error_404.html");

use crate::config::SocialConfig;

/// Build the header actions HTML with social links injected before the static buttons.
pub(crate) fn build_header_actions(social: &SocialConfig) -> String {
    let social_html = social.render_header_icons();
    // Insert social links before the theme toggle (first element in HEADER_ACTIONS_HTML)
    HEADER_ACTIONS_HTML.replacen(
        r#"<div class="oxidoc-header-actions">"#,
        &format!(r#"<div class="oxidoc-header-actions">{social_html}"#),
        1,
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
    theme: &ResolvedTheme,
) -> String {
    let project_name = &config.project.name;
    let page_title = if title.is_empty() {
        project_name.clone()
    } else {
        format!("{title} - {project_name}")
    };

    let base_url = config.project.base_url.as_deref().unwrap_or("/");
    let (logo_html, safe_name) = render_logo_html(config);

    let footer_html = render_footer(config, theme);

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
    <script src="https://cdn.jsdelivr.net/npm/iconify-icon@3.0.0/dist/iconify-icon.min.js"></script>
{css_preload}
{js_preload}
{stylesheet_link}
    {analytics_html}
    {search_head_tags}
</head>
<body data-locale="{locale}">
    <a href="#oxidoc-main" class="oxidoc-skip-nav">Skip to content</a>
    <header class="oxidoc-header" role="banner">
        {logo_html}
        {locale_switcher_html}
        {header_actions_html}
    </header>
    <div class="oxidoc-layout">
        <aside class="oxidoc-sidebar" role="navigation" aria-label="Documentation navigation">
            <div class="oxidoc-sidebar-inner">
                {sidebar_html}
            </div>
        </aside>
        <main id="oxidoc-main" class="oxidoc-content" role="main">
            {breadcrumb_html}
            <article>
                {content_html}
            </article>
            {page_meta_html}
        </main>
        <aside class="oxidoc-toc-sidebar" role="complementary" aria-label="Table of contents">
            <div class="oxidoc-toc-inner">
                {toc_html}
            </div>
        </aside>
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
        toc_html = toc_html,
        page_meta_html = page_meta_html,
        footer_html = footer_html,
        css_preload = css_preload,
        js_preload = js_preload,
        analytics_html = analytics_html,
        stylesheet_link = stylesheet_link,
        script_tag = script_tag,
        search_head_tags = search_head_tags,
        search_scripts = search_scripts,
        header_actions_html = build_header_actions(&config.social),
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
        r#"<script type="module">import mermaid from"https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";mermaid.initialize({startOnLoad:true,theme:document.documentElement.getAttribute("data-theme")==="dark"?"dark":"default"});</script>"#
    } else {
        ""
    };

    // Inject search dialog + scripts (contain curly braces, can't go in format!)
    html.replace(
        "</body>",
        &format!(
            "{}\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n{}</body>",
            SEARCH_DIALOG_HTML, THEME_TOGGLE_JS, SEARCH_DIALOG_JS, SCROLLSPY_JS, HEADER_SCROLL_JS, BACK_TO_TOP_JS, mermaid_script
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

    fn test_theme() -> crate::theme::ResolvedTheme {
        crate::theme::builtin_theme("oxidoc").unwrap()
    }

    #[test]
    fn render_page_structure_and_accessibility() {
        let config = test_config();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &config,
            "Intro",
            "<p>Hello</p>",
            "",
            "",
            "",
            "intro",
            None,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        // Essential structure
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Intro - Test Docs</title>"));
        assert!(html.contains(r#"href="/oxidoc.css""#));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-loader.js"));
        assert!(html.contains(r#"lang="en""#));
        // Accessibility and preload
        assert!(html.contains("oxidoc-skip-nav"));
        assert!(html.contains(r##"href="#oxidoc-main""##));
        assert!(html.contains(r##"id="oxidoc-main""##));
        assert!(html.contains(r#"rel="preload" href="/oxidoc.css" as="style""#));
        assert!(html.contains(r#"rel="preload" href="/oxidoc-loader.js" as="script""#));
        assert!(html.contains("oxidoc-search-trigger"));
        assert!(html.contains("oxidoc-theme-toggle"));
        // Footer present by default (attribution enabled)
        assert!(html.contains("oxidoc-footer"));

        // Logo rendering
        let logo_cfg = parse_config("[project]\nname = \"T\"\nlogo = \"/logo.svg\"").unwrap();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &logo_cfg,
            "",
            "",
            "",
            "",
            "",
            "",
            None,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains(r#"src="/logo.svg""#) && html.contains("oxidoc-logo-img"));
    }

    #[test]
    fn render_page_seo_and_description() {
        let config = test_config();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &config,
            "Test Page",
            "",
            "",
            "",
            "",
            "test",
            Some("A test page"),
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains(r#"<meta name="description" content="A test page">"#));
        assert!(html.contains(r#"<meta property="og:title""#));
        assert!(html.contains(r#"<meta property="og:type" content="article""#));
        assert!(html.contains(r#"<meta property="og:description""#));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary""#));
        assert!(html.contains("application/ld+json"));

        // Description fallback from config
        let cfg = parse_config("[project]\nname = \"T\"\ndescription = \"Fallback desc\"").unwrap();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &cfg,
            "P",
            "",
            "",
            "",
            "",
            "p",
            None,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains("Fallback desc"));
    }

    #[test]
    fn render_page_with_custom_assets_and_sri() {
        let config = test_config();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let assets = AssetConfig {
            css_path: Some("/oxidoc.a1b2c3d4.css"),
            js_path: Some("/oxidoc-loader.h5i6j7k8.js"),
            ..Default::default()
        };
        let html = render_page(
            &config,
            "Test",
            "",
            "",
            "",
            "",
            "test",
            None,
            "",
            &assets,
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains(r#"href="/oxidoc.a1b2c3d4.css""#));
        assert!(html.contains(r#"src="/oxidoc-loader.h5i6j7k8.js""#));

        let sri_assets = AssetConfig {
            css_sri: Some("sha384-abc123"),
            js_sri: Some("sha384-def456"),
            ..assets
        };
        let html = render_page(
            &config,
            "Test",
            "",
            "",
            "",
            "",
            "test",
            None,
            "",
            &sri_assets,
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
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
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &ga_config,
            "",
            "",
            "",
            "",
            "",
            "",
            None,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains("googletagmanager.com") && html.contains("G-XXXXXXXXXX"));

        let custom_config = parse_config(
            "[project]\nname = \"Test\"\n\n[analytics]\nscript = \"<script>custom</script>\"",
        )
        .unwrap();
        let i18n = default_i18n_state();
        let provider = default_search_provider();
        let html = render_page(
            &custom_config,
            "",
            "",
            "",
            "",
            "",
            "",
            None,
            "",
            &default_assets(),
            "en",
            &i18n,
            &provider,
            &test_theme(),
        );
        assert!(html.contains("custom"));
    }
}
