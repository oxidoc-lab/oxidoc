use crate::config_validate::validate_config_keys;
use crate::error::{OxidocError, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Root configuration parsed from `oxidoc.toml`.
#[derive(Debug, Deserialize)]
pub struct OxidocConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub versioning: VersioningConfig,
    #[serde(default)]
    pub i18n: I18nConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub components: ComponentsConfig,
    #[serde(default)]
    pub footer: FooterConfig,
    #[serde(default)]
    pub redirects: RedirectConfig,
    #[serde(default)]
    pub analytics: AnalyticsConfig,
    #[serde(default)]
    pub attribution: AttributionConfig,
    #[serde(default)]
    pub social: SocialConfig,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Base URL for source/edit links, e.g. "https://github.com/org/repo/blob/main/docs"
    #[serde(default)]
    pub edit_url: Option<String>,
    /// Label for the source link (default: "View page source")
    #[serde(default = "default_edit_label")]
    pub edit_label: String,
    /// When true, each statically-rendered component shows a debug outline
    /// so you can visually identify which components are static vs wasm-hydrated.
    #[serde(default)]
    pub debug_islands: bool,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub theme: String,
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: String,
    #[serde(default)]
    pub custom_css: Option<String>,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub code_font: Option<String>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
            primary: None,
            accent: None,
            dark_mode: default_dark_mode(),
            custom_css: None,
            font: None,
            code_font: None,
        }
    }
}

fn default_theme_name() -> String {
    "oxidoc".into()
}

fn default_dark_mode() -> String {
    "system".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub navigation: Vec<NavigationEntry>,
    /// Slug of the page to use as the homepage (served at `/`).
    /// Defaults to the first page in navigation.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Links displayed in the header navigation bar.
    #[serde(default)]
    pub header_links: Vec<HeaderLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HeaderLink {
    pub label: String,
    pub href: String,
}

/// A site section in the navigation. Each entry is a separate doc site
/// with its own base URL, content directory, sidebar, and optional OpenAPI spec.
#[derive(Debug, Deserialize)]
pub struct NavigationEntry {
    /// Base URL path (e.g., "/", "/api")
    #[serde(default = "default_path")]
    pub path: String,
    /// Content directory for .rdx files (e.g., "docs"). Relative to project root.
    #[serde(default)]
    pub dir: Option<String>,
    /// Sidebar groups for this section
    #[serde(default)]
    pub groups: Vec<NavigationGroup>,
    /// OpenAPI spec path — auto-generates API pages for this section
    #[serde(default)]
    pub openapi: Option<String>,
}

fn default_path() -> String {
    "/".into()
}

/// A sidebar group within a navigation entry.
#[derive(Debug, Clone, Deserialize)]
pub struct NavigationGroup {
    pub group: String,
    #[serde(default)]
    pub pages: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct VersioningConfig {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct I18nConfig {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default)]
    pub locales: Vec<String>,
    #[serde(default = "default_translation_dir")]
    pub translation_dir: String,
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            default_locale: default_locale(),
            locales: Vec::new(),
            translation_dir: default_translation_dir(),
        }
    }
}

fn default_locale() -> String {
    "en".into()
}

fn default_translation_dir() -> String {
    "i18n".into()
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default)]
    pub model_path: Option<String>,
    // Algolia preset fields
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub index_name: Option<String>,
    // Typesense preset fields
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub collection_name: Option<String>,
    // Custom provider fields
    #[serde(default)]
    pub stylesheet: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub init_script: Option<String>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model_path: None,
            app_id: None,
            api_key: None,
            index_name: None,
            host: None,
            port: None,
            protocol: None,
            collection_name: None,
            stylesheet: None,
            script: None,
            init_script: None,
        }
    }
}

fn default_provider() -> String {
    "oxidoc".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct ComponentsConfig {
    /// Maps custom tag names to JS file paths for Vanilla Web Component escape hatch.
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct FooterConfig {
    /// Copyright owner name (e.g., "Oxidoc"). Auto-generates "Copyright © {year} {owner}."
    #[serde(default)]
    pub copyright_owner: Option<String>,
    /// Optional URL for the copyright owner name.
    #[serde(default)]
    pub copyright_owner_url: Option<String>,
    #[serde(default)]
    pub links: Vec<FooterLink>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RedirectConfig {
    #[serde(default)]
    pub redirects: Vec<RedirectEntry>,
}

#[derive(Debug, Deserialize)]
pub struct RedirectEntry {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize)]
pub struct FooterLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct AnalyticsConfig {
    /// Custom analytics script tag (e.g., Google Tag Manager, Plausible)
    #[serde(default)]
    pub script: Option<String>,
    /// Google Analytics measurement ID (e.g., "G-XXXXXXXXXX")
    #[serde(default)]
    pub google_analytics: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttributionConfig {
    /// Show "Built with Oxidoc" in footer (default: true)
    #[serde(default = "default_true")]
    pub oxidoc: bool,
    /// Show theme name/author in footer (default: true)
    #[serde(default = "default_true")]
    pub theme: bool,
}

impl Default for AttributionConfig {
    fn default() -> Self {
        Self {
            oxidoc: true,
            theme: true,
        }
    }
}

pub use crate::config_social::SocialConfig;

fn default_true() -> bool {
    true
}

fn default_edit_label() -> String {
    "View page source".to_string()
}

/// Load and validate `oxidoc.toml` from a project root.
pub fn load_config(project_root: &Path) -> Result<OxidocConfig> {
    let config_path = project_root.join("oxidoc.toml");
    let content = std::fs::read_to_string(&config_path).map_err(|e| OxidocError::ConfigRead {
        path: config_path.display().to_string(),
        source: e,
    })?;

    parse_config(&content)
}

/// Parse config from a TOML string.
pub fn parse_config(content: &str) -> Result<OxidocConfig> {
    let config: OxidocConfig = toml::from_str(content).map_err(|e| OxidocError::ConfigParse {
        message: e.message().to_string(),
        source: e,
    })?;

    if config.project.name.trim().is_empty() {
        return Err(OxidocError::ConfigMissingName);
    }

    // Validate known keys and warn about unknown ones
    validate_config_keys(content);

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let toml = r#"
[project]
name = "My Docs"
"#;
        let config = parse_config(toml).unwrap();
        assert_eq!(config.project.name, "My Docs");
        assert_eq!(config.theme.theme, "oxidoc");
        assert_eq!(config.theme.primary, None);
        assert_eq!(config.theme.dark_mode, "system");
        assert_eq!(config.search.provider, "oxidoc");
        assert!(config.routing.navigation.is_empty());
        assert!(config.components.custom.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let toml = r##"
[project]
name = "My SDK Docs"
logo = "/assets/logo.svg"
description = "Complete SDK documentation"

[theme]
theme = "ocean"
primary = "#ff0000"
accent = "#00ff00"
dark_mode = "dark"
custom_css = "assets/custom.css"
font = "Georgia"
code_font = "Courier New"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Getting Started", pages = ["intro", "quickstart"] }] },
  { path = "/api", openapi = "./openapi.yaml" },
]

[versioning]
default = "v2.0"
versions = ["v1.0", "v2.0"]

[i18n]
default_locale = "en"
locales = ["en", "es", "ja"]

[search]
provider = "oxidoc"

[components.custom]
PromoBanner = "assets/js/promo-banner.js"

[[redirects.redirects]]
from = "/old-page"
to = "/new-page"
"##;
        let config = parse_config(toml).unwrap();
        assert_eq!(config.project.logo.as_deref(), Some("/assets/logo.svg"));
        assert_eq!(
            config.project.description.as_deref(),
            Some("Complete SDK documentation")
        );
        assert_eq!(config.theme.theme, "ocean");
        assert_eq!(config.theme.primary.as_deref(), Some("#ff0000"));
        assert_eq!(config.theme.accent.as_deref(), Some("#00ff00"));
        assert_eq!(config.theme.dark_mode, "dark");
        assert_eq!(
            config.theme.custom_css.as_deref(),
            Some("assets/custom.css")
        );
        assert_eq!(config.theme.font.as_deref(), Some("Georgia"));
        assert_eq!(config.theme.code_font.as_deref(), Some("Courier New"));
        assert_eq!(config.routing.navigation.len(), 2);
        assert_eq!(
            config.routing.navigation[0].groups[0].group,
            "Getting Started"
        );
        assert_eq!(
            config.routing.navigation[1].openapi.as_deref(),
            Some("./openapi.yaml")
        );
        assert_eq!(config.versioning.versions.len(), 2);
        assert_eq!(config.i18n.locales.len(), 3);
        assert_eq!(config.search.provider, "oxidoc");
        assert_eq!(
            config.components.custom.get("PromoBanner").unwrap(),
            "assets/js/promo-banner.js"
        );
        assert_eq!(config.redirects.redirects.len(), 1);
        assert_eq!(config.redirects.redirects[0].from, "/old-page");
        assert_eq!(config.redirects.redirects[0].to, "/new-page");
    }

    #[test]
    fn reject_empty_name() {
        let toml = r#"
[project]
name = "  "
"#;
        let err = parse_config(toml).unwrap_err();
        assert!(matches!(err, OxidocError::ConfigMissingName));
    }

    #[test]
    fn reject_missing_project() {
        let toml = r##"
[theme]
primary = "#ff0000"
"##;
        let err = parse_config(toml).unwrap_err();
        assert!(matches!(err, OxidocError::ConfigParse { .. }));
    }

    #[test]
    fn parse_analytics_config() {
        let toml = r##"
[project]
name = "Test Docs"

[analytics]
google_analytics = "G-XXXXXXXXXX"
script = "<script>custom analytics</script>"
"##;
        let config = parse_config(toml).unwrap();
        assert_eq!(
            config.analytics.google_analytics.as_deref(),
            Some("G-XXXXXXXXXX")
        );
        assert_eq!(
            config.analytics.script.as_deref(),
            Some("<script>custom analytics</script>")
        );
    }

    #[test]
    fn parse_analytics_optional() {
        let toml = r#"
[project]
name = "Test Docs"
"#;
        let config = parse_config(toml).unwrap();
        assert_eq!(config.analytics.google_analytics, None);
        assert_eq!(config.analytics.script, None);
    }
}
