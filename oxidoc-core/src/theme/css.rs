use super::types::ResolvedTheme;

/// Semantic CSS variables for light mode (appended to every :root block).
const SEMANTIC_VARS_LIGHT: &str = r#"    --oxidoc-success: #10b981;
    --oxidoc-success-text: #059669;
    --oxidoc-warning: #f59e0b;
    --oxidoc-warning-text: #b45309;
    --oxidoc-error: #ef4444;
    --oxidoc-error-text: #dc2626;
    --oxidoc-info: #3b82f6;
    --oxidoc-info-text: #2563eb;
    --oxidoc-new: #8b5cf6;
    --oxidoc-new-text: #7c3aed;
    --oxidoc-deprecated: #6b7280;
    --oxidoc-deprecated-text: #4b5563;
    --oxidoc-text-muted: #6b7280;
    --oxidoc-bg-subtle: #f3f4f6;
    --oxidoc-on-primary: #fff;
    --oxidoc-shadow: rgba(0, 0, 0, 0.1);
    --oxidoc-overlay: rgba(0, 0, 0, 0.4);
    --oxidoc-primary-light: color-mix(in srgb, var(--oxidoc-primary) 70%, #fff);
    --oxidoc-primary-dark: color-mix(in srgb, var(--oxidoc-primary) 70%, #000);
    --oxidoc-primary-lighter: color-mix(in srgb, var(--oxidoc-primary) 40%, #fff);
    --oxidoc-primary-darker: color-mix(in srgb, var(--oxidoc-primary) 40%, #000);
    --oxidoc-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05);
    --oxidoc-shadow-md: 0 4px 12px rgba(0, 0, 0, 0.1);
    --oxidoc-shadow-lg: 0 12px 36px rgba(0, 0, 0, 0.15);
    --oxidoc-z-header: 100;
    --oxidoc-z-sidebar: 90;
    --oxidoc-z-overlay: 1000;
    --oxidoc-z-tooltip: 10;
    --oxidoc-z-back-to-top: 50;
    --oxidoc-z-skip-nav: 200;
    --oxidoc-transition-fast: 0.15s ease;
    --oxidoc-transition-normal: 0.25s ease;
    --oxidoc-transition-slow: 0.4s ease;
    --oxidoc-transition-spring: 0.5s cubic-bezier(0.19, 1, 0.22, 1);"#;

/// Semantic CSS variables for dark mode (overrides for dark backgrounds).
const SEMANTIC_VARS_DARK: &str = r#"    --oxidoc-success: #34d399;
    --oxidoc-success-text: #6ee7b7;
    --oxidoc-warning: #fbbf24;
    --oxidoc-warning-text: #fcd34d;
    --oxidoc-error: #f87171;
    --oxidoc-error-text: #fca5a5;
    --oxidoc-info: #60a5fa;
    --oxidoc-info-text: #93c5fd;
    --oxidoc-new: #a78bfa;
    --oxidoc-new-text: #c4b5fd;
    --oxidoc-deprecated: #9ca3af;
    --oxidoc-deprecated-text: #d1d5db;
    --oxidoc-text-muted: #9ca3af;
    --oxidoc-bg-subtle: #1e293b;
    --oxidoc-on-primary: #fff;
    --oxidoc-shadow: rgba(0, 0, 0, 0.3);
    --oxidoc-overlay: rgba(0, 0, 0, 0.6);
    --oxidoc-primary-light: color-mix(in srgb, var(--oxidoc-primary) 70%, #1e293b);
    --oxidoc-primary-dark: color-mix(in srgb, var(--oxidoc-primary) 70%, #000);
    --oxidoc-primary-lighter: color-mix(in srgb, var(--oxidoc-primary) 40%, #1e293b);
    --oxidoc-primary-darker: color-mix(in srgb, var(--oxidoc-primary) 40%, #000);
    --oxidoc-shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.15);
    --oxidoc-shadow-md: 0 4px 12px rgba(0, 0, 0, 0.25);
    --oxidoc-shadow-lg: 0 12px 36px rgba(0, 0, 0, 0.4);
    --oxidoc-z-header: 100;
    --oxidoc-z-sidebar: 90;
    --oxidoc-z-overlay: 1000;
    --oxidoc-z-tooltip: 10;
    --oxidoc-z-back-to-top: 50;
    --oxidoc-z-skip-nav: 200;
    --oxidoc-transition-fast: 0.15s ease;
    --oxidoc-transition-normal: 0.25s ease;
    --oxidoc-transition-slow: 0.4s ease;
    --oxidoc-transition-spring: 0.5s cubic-bezier(0.19, 1, 0.22, 1);"#;

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
{SEMANTIC_VARS_DARK}
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
{SEMANTIC_VARS_LIGHT}
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
{SEMANTIC_VARS_LIGHT}
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
{SEMANTIC_VARS_DARK}
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
{SEMANTIC_VARS_DARK}
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
{SEMANTIC_VARS_LIGHT}
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
    use crate::theme::default_theme;

    #[test]
    fn render_css_variables_system_mode() {
        let theme = default_theme();
        let css = render_css_variables(&theme, "system");
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains("--oxidoc-accent: #f59e0b"));
        assert!(css.contains("--oxidoc-success: #10b981"));
        assert!(css.contains("--oxidoc-overlay: rgba(0, 0, 0, 0.4)"));
        assert!(css.contains("prefers-color-scheme: dark"));
        assert!(css.contains("data-theme=\"dark\""));
        assert!(css.contains("data-theme=\"light\""));
        assert!(css.contains("--oxidoc-primary-light:"));
        assert!(css.contains("--oxidoc-shadow-sm:"));
        assert!(css.contains("--oxidoc-z-header:"));
        assert!(css.contains("--oxidoc-transition-fast:"));
    }

    #[test]
    fn render_css_variables_dark_mode() {
        let theme = default_theme();
        let css = render_css_variables(&theme, "dark");
        assert!(css.contains("--oxidoc-primary: #3b82f6"));
        assert!(css.contains("--oxidoc-success: #34d399"));
        assert!(css.contains("color-scheme: dark"));
        assert!(!css.contains("prefers-color-scheme"));
    }

    #[test]
    fn render_css_variables_light_mode() {
        let theme = default_theme();
        let css = render_css_variables(&theme, "light");
        assert!(css.contains("--oxidoc-primary: #2563eb"));
        assert!(css.contains("--oxidoc-success: #10b981"));
        assert!(css.contains("color-scheme: light"));
        assert!(!css.contains("prefers-color-scheme"));
    }
}
