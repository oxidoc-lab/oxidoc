//! Translation bundle wrapping Fluent for per-locale message resolution.

use crate::error::{OxidocError, Result};
use fluent::FluentBundle;
use fluent::FluentResource;
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

/// A bundle of loaded translations for a single locale.
pub struct TranslationBundle {
    /// The locale this bundle represents (e.g., "en", "ja")
    pub locale: String,
    /// Fluent bundle containing all translations
    bundle: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for TranslationBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TranslationBundle")
            .field("locale", &self.locale)
            .finish()
    }
}

/// Fallback language identifier for when locale parsing fails.
const FALLBACK_LOCALE: &str = "en";

impl TranslationBundle {
    /// Load a translation bundle from a Fluent resource string.
    pub fn from_fluent(locale: &str, ftl_content: &str) -> Result<Self> {
        let resource = match FluentResource::try_new(ftl_content.to_string()) {
            Ok(res) => res,
            Err((_partial, errors)) => {
                let message: String = errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(OxidocError::TranslationParse {
                    path: format!("{}.ftl", locale),
                    message,
                });
            }
        };

        let lang_id: LanguageIdentifier = locale.parse().unwrap_or_else(|_| {
            FALLBACK_LOCALE
                .parse()
                .expect("hardcoded fallback locale 'en' must parse")
        });

        let mut bundle = FluentBundle::new(vec![lang_id]);
        bundle.add_resource(resource).map_err(|errors| {
            let message = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ");
            OxidocError::TranslationParse {
                path: format!("{}.ftl", locale),
                message,
            }
        })?;

        Ok(Self {
            locale: locale.to_string(),
            bundle,
        })
    }

    /// Get a translation message by key.
    ///
    /// Returns the translated string, or the key itself if not found.
    pub fn get_message(&self, key: &str) -> String {
        match self.bundle.get_message(key) {
            Some(message) => {
                if let Some(pattern) = message.value() {
                    let mut errors = vec![];
                    self.bundle
                        .format_pattern(pattern, None, &mut errors)
                        .to_string()
                } else {
                    key.to_string()
                }
            }
            None => key.to_string(),
        }
    }

    /// Get a translation message by key with arguments.
    pub fn get_message_with_args(&self, key: &str, args: &HashMap<String, String>) -> String {
        match self.bundle.get_message(key) {
            Some(message) => {
                if let Some(pattern) = message.value() {
                    let mut errors = vec![];
                    let mut fluent_args = fluent::FluentArgs::new();
                    for (k, v) in args {
                        fluent_args.set(k.clone(), v.clone());
                    }
                    self.bundle
                        .format_pattern(pattern, Some(&fluent_args), &mut errors)
                        .to_string()
                } else {
                    key.to_string()
                }
            }
            None => key.to_string(),
        }
    }

    /// Extract all message keys and their values as a map for JSON serialization.
    pub fn extract_messages(&self) -> HashMap<String, String> {
        let mut messages = HashMap::new();
        // FluentBundle doesn't expose an iterator over message IDs directly,
        // so we rely on the caller to know which keys exist, or we return
        // what we can access. For client-side bundles, we serialize the
        // raw FTL content instead.
        let _ = &self.locale; // Acknowledge usage
        messages.insert("_locale".to_string(), self.locale.clone());
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_bundle_simple() {
        let ftl = "hello = Hello\nbye = Goodbye";
        let bundle = TranslationBundle::from_fluent("en", ftl).unwrap();
        assert_eq!(bundle.get_message("hello"), "Hello");
        assert_eq!(bundle.get_message("bye"), "Goodbye");
        assert_eq!(bundle.get_message("nonexistent"), "nonexistent");
    }

    #[test]
    fn test_translation_bundle_with_variables() {
        let ftl = "greeting = Hello, {$name}!";
        let bundle = TranslationBundle::from_fluent("en", ftl).unwrap();
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        let result = bundle.get_message_with_args("greeting", &args);
        assert!(result.contains("Hello") && result.contains("World"));
    }
}
