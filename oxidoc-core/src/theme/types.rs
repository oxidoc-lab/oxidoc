use crate::error::{OxidocError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A fully resolved theme with light and dark palettes.
#[derive(Debug, Clone)]
pub struct ResolvedTheme {
    pub name: String,
    pub author: Option<String>,
    pub url: Option<String>,
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

/// Parsed theme file structure.
#[derive(Debug, Deserialize)]
struct ThemeFile {
    #[serde(default)]
    meta: ThemeMeta,
    #[serde(default)]
    colors: ThemeColors,
    #[serde(default)]
    fonts: Option<FontConfig>,
    #[serde(default)]
    radius: Option<RadiusConfig>,
    #[serde(default)]
    spacing: Option<SpacingConfig>,
}

#[derive(Debug, Default, Deserialize)]
struct ThemeMeta {
    name: Option<String>,
    author: Option<String>,
    url: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ThemeColors {
    light: Option<ColorPalette>,
    dark: Option<ColorPalette>,
}

/// Resolve a theme from config. Handles built-in themes or file paths.
pub fn resolve_theme(
    theme_name: &str,
    primary_override: Option<&str>,
    accent_override: Option<&str>,
    font_override: Option<&str>,
    code_font_override: Option<&str>,
    project_root: &Path,
) -> Result<ResolvedTheme> {
    let mut theme = if let Some(builtin) = super::builtin_theme(theme_name) {
        builtin
    } else {
        let theme_path = if Path::new(theme_name).is_absolute() {
            Path::new(theme_name).to_path_buf()
        } else {
            project_root.join(theme_name)
        };
        load_theme_file(&theme_path)?
    };

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

    Ok(theme)
}

/// Load a theme from a TOML file.
fn load_theme_file(path: &Path) -> Result<ResolvedTheme> {
    let content = std::fs::read_to_string(path).map_err(|e| OxidocError::ThemeFileRead {
        path: path.display().to_string(),
        source: e,
    })?;

    let theme_file: ThemeFile = toml::from_str(&content).map_err(|e| OxidocError::ThemeParse {
        path: path.display().to_string(),
        message: e.message().to_string(),
    })?;

    let name = theme_file.meta.name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("custom")
            .to_string()
    });

    let colors_light = theme_file
        .colors
        .light
        .ok_or_else(|| OxidocError::ThemeParse {
            path: path.display().to_string(),
            message: "Missing [colors.light] section".to_string(),
        })?;

    let colors_dark = theme_file
        .colors
        .dark
        .ok_or_else(|| OxidocError::ThemeParse {
            path: path.display().to_string(),
            message: "Missing [colors.dark] section".to_string(),
        })?;

    let fonts = theme_file.fonts.unwrap_or_else(default_fonts);
    let radius = theme_file.radius.unwrap_or_else(default_radius);
    let spacing = theme_file.spacing.unwrap_or_else(default_spacing);

    Ok(ResolvedTheme {
        name,
        author: theme_file.meta.author,
        url: theme_file.meta.url,
        colors_light,
        colors_dark,
        fonts,
        radius,
        spacing,
    })
}

pub(crate) fn default_fonts() -> FontConfig {
    FontConfig {
        sans: r#"-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif"#.to_string(),
        mono: r#""SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", Menlo, Consolas, monospace"#.to_string(),
    }
}

pub(crate) fn default_radius() -> RadiusConfig {
    RadiusConfig {
        small: "0.25rem".to_string(),
        medium: "0.375rem".to_string(),
        large: "0.5rem".to_string(),
    }
}

pub(crate) fn default_spacing() -> SpacingConfig {
    SpacingConfig {
        content_max: "48rem".to_string(),
        sidebar_width: "16rem".to_string(),
        toc_width: "14rem".to_string(),
        header_height: "3.5rem".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_builtin_theme_returns_none() {
        assert!(super::super::builtin_theme("nonexistent").is_none());
    }

    #[test]
    fn apply_primary_override() {
        let theme =
            resolve_theme("oxidoc", Some("#ff0000"), None, None, None, Path::new(".")).unwrap();
        assert_eq!(theme.colors_light.primary, "#ff0000");
        assert_eq!(theme.colors_dark.primary, "#ff0000");
    }

    #[test]
    fn apply_accent_override() {
        let theme =
            resolve_theme("oxidoc", None, Some("#00ff00"), None, None, Path::new(".")).unwrap();
        assert_eq!(theme.colors_light.accent, "#00ff00");
        assert_eq!(theme.colors_dark.accent, "#00ff00");
    }

    #[test]
    fn apply_font_override() {
        let theme =
            resolve_theme("oxidoc", None, None, Some("Georgia"), None, Path::new(".")).unwrap();
        assert_eq!(theme.fonts.sans, "Georgia");
    }

    #[test]
    fn apply_code_font_override() {
        let theme = resolve_theme(
            "oxidoc",
            None,
            None,
            None,
            Some("Courier New"),
            Path::new("."),
        )
        .unwrap();
        assert_eq!(theme.fonts.mono, "Courier New");
    }
}
