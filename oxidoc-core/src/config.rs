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
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary: default_primary(),
            dark_mode: default_dark_mode(),
        }
    }
}

fn default_primary() -> String {
    "#3b82f6".into()
}

fn default_dark_mode() -> String {
    "system".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct RoutingConfig {
    #[serde(default)]
    pub navigation: Vec<NavigationGroup>,
}

#[derive(Debug, Deserialize)]
pub struct NavigationGroup {
    pub group: String,
    #[serde(default)]
    pub pages: Vec<String>,
    #[serde(default)]
    pub openapi: Option<String>,
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
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            default_locale: default_locale(),
            locales: Vec::new(),
        }
    }
}

fn default_locale() -> String {
    "en".into()
}

#[derive(Debug, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
        }
    }
}

fn default_provider() -> String {
    "oxidoc-boostr".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct ComponentsConfig {
    /// Maps custom tag names to JS file paths for Vanilla Web Component escape hatch.
    #[serde(default)]
    pub custom: HashMap<String, String>,
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
        assert_eq!(config.theme.primary, "#3b82f6");
        assert_eq!(config.theme.dark_mode, "system");
        assert_eq!(config.search.provider, "oxidoc-boostr");
        assert!(config.routing.navigation.is_empty());
        assert!(config.components.custom.is_empty());
    }

    #[test]
    fn parse_full_config() {
        let toml = r##"
[project]
name = "My SDK Docs"
logo = "/assets/logo.svg"

[theme]
primary = "#ff0000"
dark_mode = "dark"

[routing]
navigation = [
  { group = "Getting Started", pages = ["intro", "quickstart"] },
  { group = "API Reference", openapi = "./openapi.yaml" }
]

[versioning]
default = "v2.0"
versions = ["v1.0", "v2.0"]

[i18n]
default_locale = "en"
locales = ["en", "es", "ja"]

[search]
provider = "oxidoc-tantivy"

[components.custom]
PromoBanner = "assets/js/promo-banner.js"
"##;
        let config = parse_config(toml).unwrap();
        assert_eq!(config.project.logo.as_deref(), Some("/assets/logo.svg"));
        assert_eq!(config.theme.primary, "#ff0000");
        assert_eq!(config.routing.navigation.len(), 2);
        assert_eq!(config.routing.navigation[0].group, "Getting Started");
        assert_eq!(
            config.routing.navigation[1].openapi.as_deref(),
            Some("./openapi.yaml")
        );
        assert_eq!(config.versioning.versions.len(), 2);
        assert_eq!(config.i18n.locales.len(), 3);
        assert_eq!(config.search.provider, "oxidoc-tantivy");
        assert_eq!(
            config.components.custom.get("PromoBanner").unwrap(),
            "assets/js/promo-banner.js"
        );
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
}
