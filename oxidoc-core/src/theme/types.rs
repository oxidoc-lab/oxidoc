use serde::{Deserialize, Serialize};

/// A fully resolved theme with light and dark palettes.
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub colors_light: ColorPalette,
    pub colors_dark: ColorPalette,
    pub fonts: FontConfig,
    pub radius: RadiusConfig,
    pub spacing: SpacingConfig,
}

/// Color palette for a single mode (light or dark).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub bg: String,
    pub bg_secondary: String,
    pub text: String,
    pub text_secondary: String,
    pub border: String,
    pub code_bg: String,
    pub primary: String,
    pub accent: String,
}

/// Font configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub sans: String,
    pub mono: String,
}

/// Border radius configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusConfig {
    pub small: String,
    pub medium: String,
    pub large: String,
}

/// Spacing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingConfig {
    pub content_max: String,
    pub sidebar_width: String,
    pub toc_width: String,
    pub header_height: String,
}

/// Resolve a theme from convenience config overrides.
/// Starts from the default theme and applies any overrides.
pub fn resolve_theme(
    primary_override: Option<&str>,
    accent_override: Option<&str>,
    font_override: Option<&str>,
    code_font_override: Option<&str>,
) -> ResolvedTheme {
    let mut theme = super::default_theme();

    if let Some(primary) = primary_override {
        theme.colors_light.primary = primary.to_string();
        theme.colors_dark.primary = primary.to_string();
    }

    if let Some(accent) = accent_override {
        theme.colors_light.accent = accent.to_string();
        theme.colors_dark.accent = accent.to_string();
    }

    if let Some(font) = font_override {
        theme.fonts.sans = font.to_string();
    }

    if let Some(code_font) = code_font_override {
        theme.fonts.mono = code_font.to_string();
    }

    theme
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_primary_override() {
        let theme = resolve_theme(Some("#ff0000"), None, None, None);
        assert_eq!(theme.colors_light.primary, "#ff0000");
        assert_eq!(theme.colors_dark.primary, "#ff0000");
    }

    #[test]
    fn apply_accent_override() {
        let theme = resolve_theme(None, Some("#00ff00"), None, None);
        assert_eq!(theme.colors_light.accent, "#00ff00");
        assert_eq!(theme.colors_dark.accent, "#00ff00");
    }

    #[test]
    fn apply_font_override() {
        let theme = resolve_theme(None, None, Some("Georgia"), None);
        assert_eq!(theme.fonts.sans, "Georgia");
    }

    #[test]
    fn apply_code_font_override() {
        let theme = resolve_theme(None, None, None, Some("Courier New"));
        assert_eq!(theme.fonts.mono, "Courier New");
    }
}
