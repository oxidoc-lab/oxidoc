use serde::Deserialize;
use std::collections::HashMap;

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
}

#[derive(Debug, Default, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: String,
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

#[derive(Debug, Default, Deserialize)]
pub struct I18nConfig {
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default)]
    pub locales: Vec<String>,
}

fn default_locale() -> String {
    "en".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
}

fn default_provider() -> String {
    "oxidoc-boostr".into()
}

#[derive(Debug, Default, Deserialize)]
pub struct ComponentsConfig {
    /// Maps custom tag names to JS file paths for Vanilla Web Component escape hatch.
    /// e.g., `PromoBanner = "assets/js/promo-banner.js"`
    #[serde(default)]
    pub custom: HashMap<String, String>,
}
