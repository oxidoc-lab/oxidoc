use crate::theme::ResolvedTheme;

use super::{components, search, syntax, theme};

/// Generate the base CSS for an Oxidoc site.
pub fn generate_base_css(
    resolved_theme: &ResolvedTheme,
    dark_mode: &str,
    custom_css: Option<&str>,
) -> String {
    let theme_vars = crate::theme::render_css_variables(resolved_theme, dark_mode);

    let mut css = format!(
        r#"{theme_vars}

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
{SEARCH_DIALOG_CSS}
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
        SEARCH_DIALOG_CSS = search::SEARCH_DIALOG,
        RESPONSIVE_AND_PRINT_CSS = theme::RESPONSIVE_AND_PRINT,
    );

    if let Some(custom) = custom_css {
        css.push('\n');
        css.push_str(custom);
    }

    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_base_css() {
        let theme = crate::theme::builtin_theme("oxidoc").unwrap();
        let css = generate_base_css(&theme, "system", None);
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains(".oxidoc-layout"));
        assert!(css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn generates_dark_mode_css() {
        let theme = crate::theme::builtin_theme("oxidoc").unwrap();
        let css = generate_base_css(&theme, "dark", None);
        assert!(css.contains("color-scheme: dark"));
    }

    #[test]
    fn generates_light_mode_css() {
        let theme = crate::theme::builtin_theme("oxidoc").unwrap();
        let css = generate_base_css(&theme, "light", None);
        assert!(css.contains("color-scheme: light"));
    }

    #[test]
    fn includes_custom_css() {
        let theme = crate::theme::builtin_theme("oxidoc").unwrap();
        let custom = "/* Custom styles */\nbody { margin: 10px; }";
        let css = generate_base_css(&theme, "system", Some(custom));
        assert!(css.contains(custom));
    }
}
