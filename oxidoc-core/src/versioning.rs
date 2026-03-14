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

    /// Create versioning state from config, auto-discovering archives.
    ///
    /// If `[versioning]` is not configured but archives exist, versioning is
    /// automatically enabled with "latest" as the current (default) version.
    pub fn from_config_with_archives(
        config: &VersioningConfig,
        project_root: &std::path::Path,
        current_label: Option<&str>,
    ) -> Self {
        let archived = crate::archive::discover_archives(project_root);

        if config.versions.is_empty() && archived.is_empty() {
            return Self {
                versions: Vec::new(),
                default_version: String::new(),
                enabled: false,
            };
        }

        // If user configured versions explicitly, use those
        if !config.versions.is_empty() {
            return Self::from_config(config);
        }

        // Auto-discover: archived versions + current
        let current = current_label.unwrap_or("latest").to_string();
        let mut versions = archived; // already sorted descending from discover_archives
        versions.insert(0, current.clone());

        Self {
            versions,
            default_version: current,
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
    /// The switcher preserves the current page path when switching versions.
    pub fn render_version_switcher(&self, current_version: &str) -> String {
        if !self.enabled || self.versions.len() < 2 {
            return String::new();
        }

        let versions_json = format!(
            "[{}]",
            self.versions
                .iter()
                .map(|v| format!("\"{}\"", v))
                .collect::<Vec<_>>()
                .join(",")
        );

        let chevron_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2 4l3 3 3-3"/></svg>"#;

        let mut html = format!(
            "<div class=\"oxidoc-version-switcher\" data-versions='{versions_json}'><button class=\"oxidoc-version-switcher-toggle\" aria-label=\"Select documentation version\" aria-expanded=\"false\" onclick=\"this.parentElement.classList.toggle('open');this.setAttribute('aria-expanded',this.parentElement.classList.contains('open'))\">{current_version} {chevron_svg}</button><ul class=\"oxidoc-version-switcher-menu\">",
        );

        for version in &self.versions {
            let active = if version == current_version {
                " active"
            } else {
                ""
            };
            let url = if version == &self.default_version {
                "/".to_string()
            } else {
                format!("/{}/", version)
            };
            html.push_str(&format!(
                "<li><a href=\"{url}\" class=\"oxidoc-version-link{active}\" data-version=\"{version}\" onclick=\"(function(a,e){{e.preventDefault();var p=location.pathname.replace(/^\\/+|\\/$/g,'');if(p.endsWith('.html'))p=p.slice(0,-5);var parts=p?p.split('/'):[]; var vs=JSON.parse(a.closest('.oxidoc-version-switcher').dataset.versions);if(parts.length>0&&vs.indexOf(parts[0])!==-1)parts.shift();var slug=parts.join('/');var base=a.getAttribute('href').replace(/\\/+$/,'');window.location.href=base+(slug?'/'+slug:'/')}})(this,event)\">{version}</a></li>",
            ));
        }

        html.push_str("</ul><script>(function(){var d=document.querySelector('.oxidoc-version-switcher');if(!d)return;document.addEventListener('click',function(e){if(!d.contains(e.target))d.classList.remove('open')})})()</script></div>");
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
        // Current version shown in toggle button
        assert!(html.contains(">v1.0 <"));
    }

    #[test]
    fn test_render_version_switcher_current_active() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string(), "v2.0".to_string()],
            default_version: "v2.0".to_string(),
            enabled: true,
        };
        let html = state.render_version_switcher("v2.0");
        // v2.0 should be active, and its URL should be "/"
        assert!(html.contains(r#"href="/" class="oxidoc-version-link active""#));
        // Toggle shows current version
        assert!(html.contains(">v2.0 <"));
    }

    #[test]
    fn test_render_version_switcher_urls() {
        let state = VersioningState {
            versions: vec!["v1.0".to_string(), "v2.0".to_string(), "v3.0".to_string()],
            default_version: "v3.0".to_string(),
            enabled: true,
        };
        let html = state.render_version_switcher("v1.0");
        // Non-default versions get /version/ URLs
        assert!(html.contains(r#"href="/v1.0/""#));
        assert!(html.contains(r#"href="/v2.0/""#));
        // Default version gets "/" URL
        assert!(html.contains(r#"href="/""#));
        // v1.0 should be active
        assert!(html.contains(r#"class="oxidoc-version-link active" data-version="v1.0""#));
        // Has onclick handler
        assert!(html.contains("onclick"));
        // Has click-outside script
        assert!(html.contains("addEventListener"));
    }

    #[test]
    fn test_from_config_with_archives_no_archives_no_config() {
        let config = VersioningConfig::default();
        let tmp = tempfile::tempdir().unwrap();
        let state = VersioningState::from_config_with_archives(&config, tmp.path(), None);
        assert!(!state.enabled);
    }

    #[test]
    fn test_from_config_with_archives_auto_discovers() {
        let tmp = tempfile::tempdir().unwrap();
        let archives_dir = tmp.path().join("archives");
        std::fs::create_dir(&archives_dir).unwrap();
        std::fs::write(archives_dir.join("v1.0.rdx.archive"), b"fake").unwrap();
        std::fs::write(archives_dir.join("v2.0.rdx.archive"), b"fake").unwrap();

        let config = VersioningConfig::default();
        let state = VersioningState::from_config_with_archives(&config, tmp.path(), None);
        assert!(state.enabled);
        assert_eq!(state.versions, vec!["latest", "v2.0", "v1.0"]);
        assert_eq!(state.default_version, "latest");
    }

    #[test]
    fn test_from_config_with_archives_custom_current_label() {
        let tmp = tempfile::tempdir().unwrap();
        let archives_dir = tmp.path().join("archives");
        std::fs::create_dir(&archives_dir).unwrap();
        std::fs::write(archives_dir.join("v1.0.rdx.archive"), b"fake").unwrap();

        let config = VersioningConfig::default();
        let state = VersioningState::from_config_with_archives(&config, tmp.path(), Some("v2.0"));
        assert!(state.enabled);
        assert_eq!(state.versions, vec!["v2.0", "v1.0"]);
        assert_eq!(state.default_version, "v2.0");
    }

    #[test]
    fn test_from_config_with_archives_explicit_config_takes_precedence() {
        let tmp = tempfile::tempdir().unwrap();
        let archives_dir = tmp.path().join("archives");
        std::fs::create_dir(&archives_dir).unwrap();
        std::fs::write(archives_dir.join("v1.0.rdx.archive"), b"fake").unwrap();

        // Explicit config should override auto-discovery
        let config = VersioningConfig {
            default: Some("v3.0".to_string()),
            versions: vec!["v2.0".to_string(), "v3.0".to_string()],
        };
        let state = VersioningState::from_config_with_archives(&config, tmp.path(), None);
        assert!(state.enabled);
        assert_eq!(state.default_version, "v3.0");
        assert_eq!(state.versions, vec!["v2.0", "v3.0"]);
    }
}
