//! Translation file loading and JSON bundle generation.

use super::bundle::TranslationBundle;
use crate::error::{OxidocError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Load all translation bundles from a directory.
///
/// Expects `.ftl` files named after locales (e.g., en.ftl, es.ftl, ja.ftl).
pub fn load_translations(
    project_root: &Path,
    translation_dir: &str,
    locales: &[String],
) -> Result<HashMap<String, TranslationBundle>> {
    let mut bundles = HashMap::new();
    let i18n_path = project_root.join(translation_dir);

    if !i18n_path.exists() && locales.is_empty() {
        return Ok(bundles);
    }

    for locale in locales {
        let ftl_file = i18n_path.join(format!("{}.ftl", locale));
        if !ftl_file.exists() {
            return Err(OxidocError::TranslationNotFound {
                path: ftl_file.display().to_string(),
            });
        }

        let content = std::fs::read_to_string(&ftl_file).map_err(|e| OxidocError::FileRead {
            path: ftl_file.display().to_string(),
            source: e,
        })?;

        let bundle = TranslationBundle::from_fluent(locale, &content)?;
        bundles.insert(locale.clone(), bundle);
    }

    Ok(bundles)
}

/// Generate per-locale JSON translation bundles for client-side use.
///
/// Creates `i18n/{locale}.json` files in the output directory containing
/// translation metadata and keys, ready for Wasm islands to fetch.
pub fn generate_translation_bundles(
    bundles: &HashMap<String, TranslationBundle>,
    output_dir: &Path,
) -> Result<()> {
    let i18n_dir = output_dir.join("i18n");
    std::fs::create_dir_all(&i18n_dir).map_err(|e| OxidocError::DirCreate {
        path: i18n_dir.display().to_string(),
        source: e,
    })?;

    for (locale, bundle) in bundles {
        let messages = bundle.extract_messages();
        let json = serde_json::json!({
            "locale": locale,
            "messages": messages,
        });

        let json_file = i18n_dir.join(format!("{}.json", locale));
        let json_str =
            serde_json::to_string_pretty(&json).map_err(|e| OxidocError::TranslationParse {
                path: json_file.display().to_string(),
                message: format!("Failed to serialize translation bundle: {e}"),
            })?;

        std::fs::write(&json_file, json_str).map_err(|e| OxidocError::FileWrite {
            path: json_file.display().to_string(),
            source: e,
        })?;
    }

    Ok(())
}
