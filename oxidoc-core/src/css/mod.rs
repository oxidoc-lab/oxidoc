pub mod components;
pub mod minify;
pub mod syntax;
pub mod theme;

pub use minify::minify_css;

use crate::config::OxidocConfig;

/// Generate the base CSS for an Oxidoc site.
pub fn generate_base_css(config: &OxidocConfig) -> String {
    let primary = &config.theme.primary;
    let dark_scheme_rule = theme::dark_scheme_css(&config.theme.dark_mode);

    format!(
        r#"/* Oxidoc Base Stylesheet — generated */
:root {{
    --oxidoc-primary: {primary};
    --oxidoc-font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    --oxidoc-font-mono: "SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", Menlo, Consolas, monospace;
    --oxidoc-content-max: 48rem;
    --oxidoc-sidebar-width: 16rem;
    --oxidoc-toc-width: 14rem;
    --oxidoc-header-height: 3.5rem;
}}

{dark_scheme_rule}

{RESET_AND_BODY}
{HEADER_CSS}
{LAYOUT_CSS}
{SIDEBAR_CSS}
{CONTENT_AND_TOC_CSS}
{BREADCRUMBS_CSS}
{TYPOGRAPHY_CSS}
{SKIP_NAV_AND_HEADER_ACTIONS_CSS}
{COMPONENT_CSS}
{API_CSS}
{SYNTAX_CSS}
{RESPONSIVE_AND_PRINT_CSS}
"#,
        RESET_AND_BODY = theme::RESET_AND_BODY,
        HEADER_CSS = theme::HEADER,
        LAYOUT_CSS = theme::LAYOUT,
        SIDEBAR_CSS = theme::SIDEBAR,
        CONTENT_AND_TOC_CSS = theme::CONTENT_AND_TOC,
        BREADCRUMBS_CSS = theme::BREADCRUMBS,
        TYPOGRAPHY_CSS = components::TYPOGRAPHY,
        SKIP_NAV_AND_HEADER_ACTIONS_CSS = theme::SKIP_NAV_AND_HEADER_ACTIONS,
        COMPONENT_CSS = components::COMPONENTS,
        API_CSS = components::API,
        SYNTAX_CSS = syntax::SYNTAX,
        RESPONSIVE_AND_PRINT_CSS = theme::RESPONSIVE_AND_PRINT,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_config;

    #[test]
    fn generates_base_css() {
        let config = parse_config("[project]\nname = \"Test\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains(".oxidoc-layout"));
        assert!(css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn generates_dark_mode_css() {
        let config =
            parse_config("[project]\nname = \"T\"\n[theme]\ndark_mode = \"dark\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("color-scheme: dark"));
        // Note: syntax.rs now includes prefers-color-scheme media queries for syntax highlighting
    }

    #[test]
    fn generates_light_mode_css() {
        let config =
            parse_config("[project]\nname = \"T\"\n[theme]\ndark_mode = \"light\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("color-scheme: light"));
    }
}
