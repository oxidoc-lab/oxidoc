use crate::suggest::find_suggestion;

/// Validate config keys and warn about unknown ones.
pub(crate) fn validate_config_keys(content: &str) {
    const KNOWN_KEYS: &[&str] = &[
        "project",
        "theme",
        "routing",
        "versioning",
        "i18n",
        "search",
        "components",
        "footer",
        "redirects",
        "analytics",
        "attribution",
        "social",
        "llm",
    ];

    if let Ok(value) = toml::from_str::<toml::Table>(content) {
        for key in value.keys() {
            if !KNOWN_KEYS.contains(&key.as_str()) {
                let suggestion = find_suggestion(key, KNOWN_KEYS);
                if let Some(suggested_key) = suggestion {
                    tracing::warn!(
                        unknown_key = key,
                        suggested_key = suggested_key,
                        "Unknown config key; did you mean '{}'?",
                        suggested_key
                    );
                } else {
                    tracing::warn!(unknown_key = key, "Unknown config key in oxidoc.toml");
                }
            }
        }
    }
}
