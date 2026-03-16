use crate::config::OxidocConfig;
use crate::i18n::I18nState;
use crate::search_provider::SearchProvider;
use crate::template::{
    BACK_TO_TOP_JS, HEADER_SCROLL_JS, SEARCH_DIALOG_HTML, SEARCH_DIALOG_JS, THEME_TOGGLE_JS,
    build_header_actions, build_menu_toggle, build_mobile_nav_links, render_logo_html,
};
use crate::template_assets::{
    AssetConfig, build_preload_links, build_script_tag, build_stylesheet_link,
};
use crate::template_parts::{render_analytics_script, render_footer};

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
    <header class="oxidoc-header oxidoc-header-landing" role="banner">
        {menu_toggle_html}
        {logo_html}
        {locale_switcher_html}
        {header_actions_html}
    </header>
    <div class="oxidoc-sidebar-overlay"></div>
    <aside class="oxidoc-sidebar oxidoc-sidebar-landing" role="navigation" aria-label="Main navigation">
        <div class="oxidoc-sidebar-inner">
            {mobile_nav_links}
        </div>
    </aside>
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
        menu_toggle_html = build_menu_toggle(),
        header_actions_html = build_header_actions(&config.social),
        mobile_nav_links = build_mobile_nav_links(&config.routing.header_links),
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
        concat!(
            "<script type=\"module\">",
            include_str!("templates/mermaid_init.js"),
            "</script>"
        )
    } else {
        ""
    };

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
            "{}\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n<script>{}</script>\n{}</body>",
            search_html, THEME_TOGGLE_JS, SEARCH_DIALOG_JS, HEADER_SCROLL_JS, BACK_TO_TOP_JS, crate::template::MOBILE_MENU_JS, mermaid_script
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
        );
        assert!(html.contains("oxidoc-landing"));
        assert!(html.contains("oxidoc-header-landing"));
        assert!(html.contains("oxidoc-sidebar-landing"));
        assert!(!html.contains("oxidoc-toc-sidebar"));
        assert!(!html.contains("oxidoc-layout"));
        assert!(html.contains("<p>Hello</p>"));
        assert!(html.contains("oxidoc-footer"));
    }
}
