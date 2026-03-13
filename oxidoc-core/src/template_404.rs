use crate::config::OxidocConfig;
use crate::i18n::I18nState;
use crate::search_provider::SearchProvider;
use crate::template::{ERROR_404_HTML, build_header_actions, render_logo_html};
use crate::template_assets::{
    AssetConfig, build_preload_links, build_script_tag, build_stylesheet_link,
};
use crate::template_parts::{render_analytics_script, render_footer};
use crate::theme::ResolvedTheme;

/// Generate a 404 error page using the site template.
pub fn render_404_page(
    config: &OxidocConfig,
    assets: &AssetConfig<'_>,
    locale: &str,
    i18n_state: &I18nState,
    search_provider: &SearchProvider,
    theme: &ResolvedTheme,
) -> String {
    let (logo_html, safe_name) = render_logo_html(config);
    let footer_html = render_footer(config, theme);

    let css_href = assets.css_path.unwrap_or("/oxidoc.css");
    let js_src = assets.js_path.unwrap_or("/oxidoc-loader.js");
    let analytics_html = render_analytics_script(config);
    let stylesheet_link = build_stylesheet_link(css_href, assets.css_sri);
    let script_tag = build_script_tag(js_src, assets.js_sri);
    let (css_preload, js_preload) =
        build_preload_links(css_href, assets.css_sri, js_src, assets.js_sri);

    let search_head_tags = search_provider.render_head_tags();
    let search_scripts = search_provider.render_scripts();

    let locale_switcher_html = i18n_state.render_locale_switcher(locale, "/");

    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}" data-locale="{locale}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Page Not Found - {project_name}</title>
    <meta name="description" content="The page you are looking for could not be found.">
    <meta name="generator" content="oxidoc">
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
    <div class="oxidoc-layout oxidoc-layout--no-sidebar">
        <main id="oxidoc-main" class="oxidoc-content" role="main">
            <article>
                {ERROR_404_HTML}
            </article>
        </main>
    </div>
    {footer_html}
{script_tag}
    {search_scripts}
</body>
</html>"##,
        lang = locale,
        locale = locale,
        project_name = safe_name,
        logo_html = logo_html,
        locale_switcher_html = locale_switcher_html,
        footer_html = footer_html,
        css_preload = css_preload,
        js_preload = js_preload,
        analytics_html = analytics_html,
        stylesheet_link = stylesheet_link,
        script_tag = script_tag,
        search_head_tags = search_head_tags,
        search_scripts = search_scripts,
        header_actions_html = build_header_actions(&config.social),
        ERROR_404_HTML = ERROR_404_HTML,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config;

    fn test_config() -> OxidocConfig {
        parse_config("[project]\nname = \"Test Docs\"").unwrap()
    }

    fn default_i18n_state() -> I18nState {
        I18nState::from_config("en", &[])
    }

    fn default_search_provider() -> SearchProvider {
        SearchProvider::Oxidoc { model_path: None }
    }

    fn test_theme() -> ResolvedTheme {
        crate::theme::builtin_theme("oxidoc").unwrap()
    }

    #[test]
    fn render_404_page_contains_essentials() {
        let config = test_config();
        let html = render_404_page(
            &config,
            &AssetConfig::default(),
            "en",
            &default_i18n_state(),
            &default_search_provider(),
            &test_theme(),
        );
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("404"));
        assert!(html.contains("Not Found"));
        assert!(html.contains("Return to home"));
    }

    #[test]
    fn render_404_page_with_assets_and_sri() {
        let config = test_config();
        let assets = AssetConfig {
            css_path: Some("/oxidoc.a1b2c3d4.css"),
            js_path: Some("/oxidoc-loader.h5i6j7k8.js"),
            css_sri: Some("sha384-abc123"),
            js_sri: Some("sha384-def456"),
        };
        let html = render_404_page(
            &config,
            &assets,
            "en",
            &default_i18n_state(),
            &default_search_provider(),
            &test_theme(),
        );
        assert!(html.contains(r#"href="/oxidoc.a1b2c3d4.css""#));
        assert!(html.contains(r#"src="/oxidoc-loader.h5i6j7k8.js""#));
        assert!(html.contains(r#"integrity="sha384-abc123""#));
        assert!(html.contains(r#"integrity="sha384-def456""#));
    }
}
