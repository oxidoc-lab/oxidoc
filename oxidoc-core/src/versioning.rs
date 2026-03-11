//! Multi-version site generation.
//!
//! When versioning is configured, builds separate version directories (e.g., v1.0/, v2.0/)
//! with a version switcher included in each page.

use crate::config::VersioningConfig;

/// Resolved versioning state.
#[derive(Debug, Clone)]
pub struct VersioningState {
    /// All available versions (e.g., ["v1.0", "v2.0"])
    pub versions: Vec<String>,
    /// The default version to build at the root (e.g., "v2.0")
    pub default_version: String,
    /// Whether versioning is enabled
    pub enabled: bool,
}

impl VersioningState {
    /// Create versioning state from config.
    pub fn from_config(config: &VersioningConfig) -> Self {
        if config.versions.is_empty() {
            return Self {
                versions: Vec::new(),
                default_version: String::new(),
                enabled: false,
            };
        }

        let default_version = config
            .default
            .clone()
            .or_else(|| config.versions.last().cloned())
            .unwrap_or_else(|| "v1.0".to_string());

        Self {
            versions: config.versions.clone(),
            default_version,
            enabled: true,
        }
    }

    /// Get all subdirectories where versions should be built.
    ///
    /// If versioning is enabled, returns ["v1.0", "v2.0", ""] (empty = root).
    /// Otherwise returns [""] (root only).
    pub fn build_dirs(&self) -> Vec<String> {
        if !self.enabled {
            return vec![String::new()];
        }

        let mut dirs = self.versions.clone();
        dirs.push(String::new()); // Root gets the default version
        dirs
    }

    /// Generate HTML for a version switcher component.
    ///
    /// Returns HTML that can be injected into the page template.
    pub fn render_version_switcher(&self, current_version: &str) -> String {
        if !self.enabled || self.versions.len() < 2 {
            return String::new();
        }

        let mut html = String::from(
            r#"<div class="oxidoc-version-switcher"><select aria-label="Select documentation version">"#,
        );

        for version in &self.versions {
            let selected = if version == current_version {
                r#" selected"#
            } else {
                ""
            };
            let url = if version == &self.default_version {
                "/".to_string()
            } else {
                format!("/{}/", version)
            };
            html.push_str(&format!(
                r#"<option value="{}"{}>{}</option>"#,
                url, selected, version
            ));
        }

        html.push_str("</select></div>");
        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioning_state_disabled_by_default() {
        let config = VersioningConfig::default();
        let state = VersioningState::from_config(&config);
        assert!(!state.enabled);
        assert_eq!(state.versions.len(), 0);
    }

    #[test]
    fn test_versioning_state_enabled() {
        let config = VersioningConfig {
            default: Some("v2.0".to_string()),
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
        };
        let state = VersioningState::from_config(&config);
        assert!(state.enabled);
        assert_eq!(state.default_version, "v2.0");
        assert_eq!(state.versions.len(), 2);
    }

    #[test]
    fn test_versioning_state_default_to_latest() {
        let config = VersioningConfig {
            default: None,
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
        };
        let state = VersioningState::from_config(&config);
        assert_eq!(state.default_version, "v2.0");
    }

    #[test]
    fn test_versioning_build_dirs_disabled() {
        let state = VersioningState {
            versions: vec![],
            default_version: String::new(),
            enabled: false,
        };
        assert_eq!(state.build_dirs(), vec!["".to_string()]);
    }

    #[test]
    fn test_versioning_build_dirs_enabled() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
            default_version: "v2.0".to_string(),
            enabled: true,
        };
        let dirs = state.build_dirs();
        assert_eq!(dirs.len(), 3);
        assert!(dirs.contains(&"v1.0".to_string()));
        assert!(dirs.contains(&"v2.0".to_string()));
        assert!(dirs.contains(&"".to_string()));
    }

    #[test]
    fn test_render_version_switcher_disabled() {
        let state = VersioningState {
            versions: vec![],
            default_version: String::new(),
            enabled: false,
        };
        assert_eq!(state.render_version_switcher("v1.0"), "");
    }

    #[test]
    fn test_render_version_switcher_single_version() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string()],
            default_version: "v1.0".to_string(),
            enabled: true,
        };
        assert_eq!(state.render_version_switcher("v1.0"), "");
    }

    #[test]
    fn test_render_version_switcher_multiple_versions() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
            default_version: "v2.0".to_string(),
            enabled: true,
        };
        let html = state.render_version_switcher("v1.0");
        assert!(html.contains("oxidoc-version-switcher"));
        assert!(html.contains("v1.0"));
        assert!(html.contains("v2.0"));
        assert!(html.contains(r#"selected"#));
    }

    #[test]
    fn test_render_version_switcher_current_selected() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
            default_version: "v2.0".to_string(),
            enabled: true,
        };
        let html = state.render_version_switcher("v2.0");
        // v2.0 should be selected, and its URL should be "/"
        assert!(html.contains(r#"value="/" selected"#));
    }
}
