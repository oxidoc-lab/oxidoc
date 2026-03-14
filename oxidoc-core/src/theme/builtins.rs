use super::types::{ColorPalette, FontConfig, RadiusConfig, ResolvedTheme, SpacingConfig};

/// Get the default Oxidoc theme.
pub fn default_theme() -> ResolvedTheme {
    ResolvedTheme {
        colors_light: ColorPalette {
            bg: "#ffffff".to_string(),
            bg_secondary: "#f8fafc".to_string(),
            text: "#1e293b".to_string(),
            text_secondary: "#64748b".to_string(),
            border: "#e2e8f0".to_string(),
            code_bg: "#f1f5f9".to_string(),
            primary: "#2563eb".to_string(),
            accent: "#f59e0b".to_string(),
        },
        colors_dark: ColorPalette {
            bg: "#0f172a".to_string(),
            bg_secondary: "#1e293b".to_string(),
            text: "#e2e8f0".to_string(),
            text_secondary: "#94a3b8".to_string(),
            border: "#334155".to_string(),
            code_bg: "#1e293b".to_string(),
            primary: "#3b82f6".to_string(),
            accent: "#fbbf24".to_string(),
        },
        fonts: FontConfig {
            sans: r#"-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif"#.to_string(),
            mono: r#""SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", Menlo, Consolas, monospace"#.to_string(),
        },
        radius: RadiusConfig {
            small: "0.25rem".to_string(),
            medium: "0.375rem".to_string(),
            large: "0.5rem".to_string(),
        },
        spacing: SpacingConfig {
            content_max: "48rem".to_string(),
            sidebar_width: "16rem".to_string(),
            toc_width: "14rem".to_string(),
            header_height: "3.5rem".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_theme_has_expected_colors() {
        let theme = default_theme();
        assert_eq!(theme.colors_light.primary, "#2563eb");
        assert_eq!(theme.colors_dark.primary, "#3b82f6");
    }
}
