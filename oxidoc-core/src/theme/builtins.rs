use super::types::{
    ColorPalette, FontConfig, RadiusConfig, ResolvedTheme, SpacingConfig, default_fonts,
    default_radius, default_spacing,
};

/// Get a built-in theme by name.
pub fn builtin_theme(name: &str) -> Option<ResolvedTheme> {
    match name {
        "oxidoc" => Some(THEME_OXIDOC.clone()),
        "ocean" => Some(THEME_OCEAN.clone()),
        "forest" => Some(THEME_FOREST.clone()),
        _ => None,
    }
}

lazy_static::lazy_static! {
    /// Default "Oxidoc" theme — clean and professional.
    static ref THEME_OXIDOC: ResolvedTheme = ResolvedTheme {
        name: "oxidoc".to_string(),
        author: Some("Oxidoc".to_string()),
        url: Some("https://github.com/oxidoc-lab/oxidoc".to_string()),
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
    };

    /// "Ocean" theme — cool blues and cyans.
    static ref THEME_OCEAN: ResolvedTheme = ResolvedTheme {
        name: "ocean".to_string(),
        author: Some("Oxidoc".to_string()),
        url: Some("https://github.com/oxidoc-lab/oxidoc".to_string()),
        colors_light: ColorPalette {
            bg: "#ffffff".to_string(),
            bg_secondary: "#f0f9ff".to_string(),
            text: "#0c4a6e".to_string(),
            text_secondary: "#0369a1".to_string(),
            border: "#bae6fd".to_string(),
            code_bg: "#f0f9ff".to_string(),
            primary: "#0284c7".to_string(),
            accent: "#06b6d4".to_string(),
        },
        colors_dark: ColorPalette {
            bg: "#0c1222".to_string(),
            bg_secondary: "#162032".to_string(),
            text: "#e0f2fe".to_string(),
            text_secondary: "#7dd3fc".to_string(),
            border: "#1e3a5f".to_string(),
            code_bg: "#162032".to_string(),
            primary: "#38bdf8".to_string(),
            accent: "#22d3ee".to_string(),
        },
        fonts: default_fonts(),
        radius: default_radius(),
        spacing: default_spacing(),
    };

    /// "Forest" theme — earthy greens and warm browns.
    static ref THEME_FOREST: ResolvedTheme = ResolvedTheme {
        name: "forest".to_string(),
        author: Some("Oxidoc".to_string()),
        url: Some("https://github.com/oxidoc-lab/oxidoc".to_string()),
        colors_light: ColorPalette {
            bg: "#ffffff".to_string(),
            bg_secondary: "#f0fdf4".to_string(),
            text: "#14532d".to_string(),
            text_secondary: "#15803d".to_string(),
            border: "#bbf7d0".to_string(),
            code_bg: "#f0fdf4".to_string(),
            primary: "#16a34a".to_string(),
            accent: "#ca8a04".to_string(),
        },
        colors_dark: ColorPalette {
            bg: "#0a1a0f".to_string(),
            bg_secondary: "#14291a".to_string(),
            text: "#dcfce7".to_string(),
            text_secondary: "#86efac".to_string(),
            border: "#1a3a23".to_string(),
            code_bg: "#14291a".to_string(),
            primary: "#4ade80".to_string(),
            accent: "#facc15".to_string(),
        },
        fonts: default_fonts(),
        radius: default_radius(),
        spacing: default_spacing(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_oxidoc_theme_exists() {
        let theme = builtin_theme("oxidoc").expect("oxidoc theme should exist");
        assert_eq!(theme.name, "oxidoc");
        assert_eq!(theme.colors_light.primary, "#2563eb");
        assert_eq!(theme.colors_dark.primary, "#3b82f6");
    }

    #[test]
    fn builtin_ocean_theme_exists() {
        let theme = builtin_theme("ocean").expect("ocean theme should exist");
        assert_eq!(theme.name, "ocean");
        assert_eq!(theme.colors_light.primary, "#0284c7");
    }

    #[test]
    fn builtin_forest_theme_exists() {
        let theme = builtin_theme("forest").expect("forest theme should exist");
        assert_eq!(theme.name, "forest");
        assert_eq!(theme.colors_light.primary, "#16a34a");
    }
}
