//! I18n state management and locale routing.

use super::locales::locale_display_name;

/// Resolved i18n state from configuration.
#[derive(Debug, Clone)]
pub struct I18nState {
    /// All configured locales (e.g., ["en", "es", "ja"])
    pub locales: Vec<String>,
    /// The default locale to use for root paths (e.g., "en")
    pub default_locale: String,
    /// Whether i18n is enabled (true if more than 1 locale)
    pub enabled: bool,
}

impl I18nState {
    /// Create i18n state from configuration.
    pub fn from_config(default_locale: &str, locales: &[String]) -> Self {
        let enabled = locales.len() > 1;
        Self {
            locales: locales.to_vec(),
            default_locale: default_locale.to_string(),
            enabled,
        }
    }

    /// Get all locales where builds should be generated.
    ///
    /// If i18n is enabled, returns all locales. Otherwise returns [default_locale].
    pub fn build_locales(&self) -> Vec<String> {
        if self.enabled {
            self.locales.clone()
        } else {
            vec![self.default_locale.clone()]
        }
    }

    /// Check if a locale is the default.
    pub fn is_default_locale(&self, locale: &str) -> bool {
        locale == self.default_locale
    }

    /// Generate HTML for a locale switcher component.
    ///
    /// Returns HTML that can be injected into the page template.
    pub fn render_locale_switcher(&self, current_locale: &str, current_path: &str) -> String {
        if !self.enabled || self.locales.len() < 2 {
            return String::new();
        }

        let mut html = String::from(
            r#"<div class="oxidoc-locale-switcher"><select aria-label="Select language">"#,
        );

        for locale in &self.locales {
            let selected = if locale == current_locale {
                r#" selected"#
            } else {
                ""
            };

            let url = if self.is_default_locale(locale) {
                current_path.to_string()
            } else {
                let stripped = current_path.trim_start_matches('/').trim_end_matches('/');
                if stripped.is_empty() {
                    format!("/{}/", locale)
                } else {
                    format!("/{}/{}", locale, stripped)
                }
            };

            let display_name = locale_display_name(locale);
            html.push_str(&format!(
                r#"<option value="{}"{}>{}</option>"#,
                crate::utils::html_escape(&url),
                selected,
                crate::utils::html_escape(display_name)
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
    fn test_i18n_state_single_locale_disabled() {
        let state = I18nState::from_config("en", &[]);
        assert!(!state.enabled);
    }

    #[test]
    fn test_i18n_state_multiple_locales_enabled() {
        let locales = vec!["en".to_string(), "es".to_string()];
        let state = I18nState::from_config("en", &locales);
        assert!(state.enabled);
        assert_eq!(state.locales.len(), 2);
    }

    #[test]
    fn test_i18n_state_build_locales_disabled() {
        let state = I18nState::from_config("en", &[]);
        let build = state.build_locales();
        assert_eq!(build, vec!["en"]);
    }

    #[test]
    fn test_i18n_state_build_locales_enabled() {
        let locales = vec!["en".to_string(), "es".to_string(), "ja".to_string()];
        let state = I18nState::from_config("en", &locales);
        let build = state.build_locales();
        assert_eq!(build.len(), 3);
    }

    #[test]
    fn test_is_default_locale() {
        let state = I18nState::from_config("en", &["en".to_string(), "es".to_string()]);
        assert!(state.is_default_locale("en"));
        assert!(!state.is_default_locale("es"));
    }

    #[test]
    fn test_render_locale_switcher_disabled() {
        let state = I18nState::from_config("en", &[]);
        let html = state.render_locale_switcher("en", "/intro");
        assert_eq!(html, "");
    }

    #[test]
    fn test_render_locale_switcher_single_locale() {
        let state = I18nState::from_config("en", &["en".to_string()]);
        let html = state.render_locale_switcher("en", "/intro");
        assert_eq!(html, "");
    }

    #[test]
    fn test_render_locale_switcher_multiple_locales() {
        let state = I18nState::from_config(
            "en",
            &["en".to_string(), "es".to_string(), "ja".to_string()],
        );
        let html = state.render_locale_switcher("en", "/intro");
        assert!(html.contains("oxidoc-locale-switcher"));
        assert!(html.contains("English"));
        assert!(html.contains("Español"));
        assert!(html.contains(r#"selected"#));
    }
}
