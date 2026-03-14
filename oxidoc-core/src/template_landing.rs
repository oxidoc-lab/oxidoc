use crate::config::OxidocConfig;
use crate::i18n::I18nState;
use crate::search_provider::SearchProvider;
use crate::template::{
    BACK_TO_TOP_JS, HEADER_SCROLL_JS, SEARCH_DIALOG_HTML, SEARCH_DIALOG_JS, THEME_TOGGLE_JS,
    build_header_actions, render_logo_html,
};
use crate::template_assets::{
    AssetConfig, build_preload_links, build_script_tag, build_stylesheet_link,
};
use crate::template_parts::{render_analytics_script, render_footer};
use crate::theme::ResolvedTheme;

/// Render a full-width landing page (no sidebar, no TOC, no breadcrumbs).
#[allow(clippy::too_many_arguments)]
pub fn render_landing_page(
    config: &OxidocConfig,
    title: &str,
    content_html: &str,
    active_slug: &str,
    description: Option<&str>,
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

    let default_description = format!("{} documentation", project_name);
    let page_description = description
        .or(config.project.description.as_deref())
        .unwrap_or(default_description.as_str());

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

    let search_head_tags = search_provider.render_head_tags();
    let search_scripts = search_provider.render_scripts();

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
    <header class="oxidoc-header oxidoc-header-landing" role="banner">
        {logo_html}
        {locale_switcher_html}
        {header_actions_html}
    </header>
    <main id="oxidoc-main" class="oxidoc-landing" role="main">
        <article>
            {content_html}
        </article>
    </main>
    {footer_html}
{script_tag}
    {search_scripts}

</body>
</html>"##,
        lang = locale,
        locale = locale,
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

    let mermaid_script = if html.contains(r#"<pre class="mermaid">"#) {
        r#"<script type="module">import mermaid from"https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";mermaid.initialize({startOnLoad:true,theme:document.documentElement.getAttribute("data-theme")==="dark"?"dark":"default"});</script>"#
    } else {
        ""
    };

    html.replace(
        "</body>",
        &format!(
            "{}\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n{}</body>",
            SEARCH_DIALOG_HTML, THEME_TOGGLE_JS, SEARCH_DIALOG_JS, HEADER_SCROLL_JS, BACK_TO_TOP_JS, mermaid_script
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

    #[test]
    fn landing_page_has_no_sidebar_or_toc() {
        let config = test_config();
        let i18n = crate::i18n::I18nState::from_config("en", &[]);
        let provider = SearchProvider::Oxidoc { model_path: None };
        let theme = crate::theme::builtin_theme("oxidoc").unwrap();
        let html = render_landing_page(
            &config,
            "Welcome",
            "<p>Hello</p>",
            "",
            None,
            &AssetConfig::default(),
            "en",
            &i18n,
            &provider,
            &theme,
        );
        assert!(html.contains("oxidoc-landing"));
        assert!(html.contains("oxidoc-header-landing"));
        assert!(!html.contains("oxidoc-sidebar"));
        assert!(!html.contains("oxidoc-toc-sidebar"));
        assert!(!html.contains("oxidoc-layout"));
        assert!(!html.contains("oxidoc-header-nav"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-footer"));
    }
}
