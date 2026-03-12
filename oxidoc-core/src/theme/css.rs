use super::types::ResolvedTheme;

/// Generate CSS variables for theme.
pub fn render_css_variables(theme: &ResolvedTheme, dark_mode: &str) -> String {
    let light = &theme.colors_light;
    let dark = &theme.colors_dark;
    let fonts = &theme.fonts;
    let radius = &theme.radius;
    let spacing = &theme.spacing;

    match dark_mode {
        "dark" => {
            format!(
                r#"/* Oxidoc Theme Variables — generated */
html {{ color-scheme: dark; }}
:root {{
    --oxidoc-primary: {primary_dark};
    --oxidoc-accent: {accent_dark};
    --oxidoc-bg: {bg_dark};
    --oxidoc-bg-secondary: {bg_secondary_dark};
    --oxidoc-text: {text_dark};
    --oxidoc-text-secondary: {text_secondary_dark};
    --oxidoc-border: {border_dark};
    --oxidoc-code-bg: {code_bg_dark};
    --oxidoc-font-sans: {font_sans};
    --oxidoc-font-mono: {font_mono};
    --oxidoc-radius-sm: {radius_small};
    --oxidoc-radius-md: {radius_medium};
    --oxidoc-radius-lg: {radius_large};
    --oxidoc-content-max: {content_max};
    --oxidoc-sidebar-width: {sidebar_width};
    --oxidoc-toc-width: {toc_width};
    --oxidoc-header-height: {header_height};
}}
"#,
                primary_dark = dark.primary,
                accent_dark = dark.accent,
                bg_dark = dark.bg,
                bg_secondary_dark = dark.bg_secondary,
                text_dark = dark.text,
                text_secondary_dark = dark.text_secondary,
                border_dark = dark.border,
                code_bg_dark = dark.code_bg,
                font_sans = fonts.sans,
                font_mono = fonts.mono,
                radius_small = radius.small,
                radius_medium = radius.medium,
                radius_large = radius.large,
                content_max = spacing.content_max,
                sidebar_width = spacing.sidebar_width,
                toc_width = spacing.toc_width,
                header_height = spacing.header_height,
            )
        }
        "light" => {
            format!(
                r#"/* Oxidoc Theme Variables — generated */
html {{ color-scheme: light; }}
:root {{
    --oxidoc-primary: {primary_light};
    --oxidoc-accent: {accent_light};
    --oxidoc-bg: {bg_light};
    --oxidoc-bg-secondary: {bg_secondary_light};
    --oxidoc-text: {text_light};
    --oxidoc-text-secondary: {text_secondary_light};
    --oxidoc-border: {border_light};
    --oxidoc-code-bg: {code_bg_light};
    --oxidoc-font-sans: {font_sans};
    --oxidoc-font-mono: {font_mono};
    --oxidoc-radius-sm: {radius_small};
    --oxidoc-radius-md: {radius_medium};
    --oxidoc-radius-lg: {radius_large};
    --oxidoc-content-max: {content_max};
    --oxidoc-sidebar-width: {sidebar_width};
    --oxidoc-toc-width: {toc_width};
    --oxidoc-header-height: {header_height};
}}
"#,
                primary_light = light.primary,
                accent_light = light.accent,
                bg_light = light.bg,
                bg_secondary_light = light.bg_secondary,
                text_light = light.text,
                text_secondary_light = light.text_secondary,
                border_light = light.border,
                code_bg_light = light.code_bg,
                font_sans = fonts.sans,
                font_mono = fonts.mono,
                radius_small = radius.small,
                radius_medium = radius.medium,
                radius_large = radius.large,
                content_max = spacing.content_max,
                sidebar_width = spacing.sidebar_width,
                toc_width = spacing.toc_width,
                header_height = spacing.header_height,
            )
        }
        _ => {
            // System mode: default to light, with dark variant in media query and manual toggle
            format!(
                r#"/* Oxidoc Theme Variables — generated */
:root {{
    --oxidoc-primary: {primary_light};
    --oxidoc-accent: {accent_light};
    --oxidoc-bg: {bg_light};
    --oxidoc-bg-secondary: {bg_secondary_light};
    --oxidoc-text: {text_light};
    --oxidoc-text-secondary: {text_secondary_light};
    --oxidoc-border: {border_light};
    --oxidoc-code-bg: {code_bg_light};
    --oxidoc-font-sans: {font_sans};
    --oxidoc-font-mono: {font_mono};
    --oxidoc-radius-sm: {radius_small};
    --oxidoc-radius-md: {radius_medium};
    --oxidoc-radius-lg: {radius_large};
    --oxidoc-content-max: {content_max};
    --oxidoc-sidebar-width: {sidebar_width};
    --oxidoc-toc-width: {toc_width};
    --oxidoc-header-height: {header_height};
}}

@media (prefers-color-scheme: dark) {{
    :root {{
        --oxidoc-primary: {primary_dark};
        --oxidoc-accent: {accent_dark};
        --oxidoc-bg: {bg_dark};
        --oxidoc-bg-secondary: {bg_secondary_dark};
        --oxidoc-text: {text_dark};
        --oxidoc-text-secondary: {text_secondary_dark};
        --oxidoc-border: {border_dark};
        --oxidoc-code-bg: {code_bg_dark};
    }}
}}

html[data-theme="dark"] {{
    --oxidoc-primary: {primary_dark};
    --oxidoc-accent: {accent_dark};
    --oxidoc-bg: {bg_dark};
    --oxidoc-bg-secondary: {bg_secondary_dark};
    --oxidoc-text: {text_dark};
    --oxidoc-text-secondary: {text_secondary_dark};
    --oxidoc-border: {border_dark};
    --oxidoc-code-bg: {code_bg_dark};
}}

html[data-theme="light"] {{
    --oxidoc-primary: {primary_light};
    --oxidoc-accent: {accent_light};
    --oxidoc-bg: {bg_light};
    --oxidoc-bg-secondary: {bg_secondary_light};
    --oxidoc-text: {text_light};
    --oxidoc-text-secondary: {text_secondary_light};
    --oxidoc-border: {border_light};
    --oxidoc-code-bg: {code_bg_light};
}}
"#,
                primary_light = light.primary,
                accent_light = light.accent,
                bg_light = light.bg,
                bg_secondary_light = light.bg_secondary,
                text_light = light.text,
                text_secondary_light = light.text_secondary,
                border_light = light.border,
                code_bg_light = light.code_bg,
                primary_dark = dark.primary,
                accent_dark = dark.accent,
                bg_dark = dark.bg,
                bg_secondary_dark = dark.bg_secondary,
                text_dark = dark.text,
                text_secondary_dark = dark.text_secondary,
                border_dark = dark.border,
                code_bg_dark = dark.code_bg,
                font_sans = fonts.sans,
                font_mono = fonts.mono,
                radius_small = radius.small,
                radius_medium = radius.medium,
                radius_large = radius.large,
                content_max = spacing.content_max,
                sidebar_width = spacing.sidebar_width,
                toc_width = spacing.toc_width,
                header_height = spacing.header_height,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::builtin_theme;

    #[test]
    fn render_css_variables_system_mode() {
        let theme = builtin_theme("oxidoc").unwrap();
        let css = render_css_variables(&theme, "system");
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains("--oxidoc-accent: #f59e0b"));
        assert!(css.contains("prefers-color-scheme: dark"));
        assert!(css.contains("data-theme=\"dark\""));
        assert!(css.contains("data-theme=\"light\""));
    }

    #[test]
    fn render_css_variables_dark_mode() {
        let theme = builtin_theme("oxidoc").unwrap();
        let css = render_css_variables(&theme, "dark");
        assert!(css.contains("--oxidoc-primary: #3b82f6"));
        assert!(css.contains("color-scheme: dark"));
        assert!(!css.contains("prefers-color-scheme"));
    }

    #[test]
    fn render_css_variables_light_mode() {
        let theme = builtin_theme("oxidoc").unwrap();
        let css = render_css_variables(&theme, "light");
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains("color-scheme: light"));
        assert!(!css.contains("prefers-color-scheme"));
    }
}
