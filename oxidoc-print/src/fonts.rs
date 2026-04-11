use oxipdf::ir::style::typography::FontStyle;
use oxipdf::shaping::provider::{BytesFontProvider, FontProvider};
use std::path::Path;
use std::sync::Arc;

/// Build a `FontProvider` by scanning system font directories and any additional paths.
///
/// Only loads fonts that match the requested families to avoid excessive memory use.
/// Returns `None` if no fonts could be found at all.
pub fn build_font_provider(extra_dirs: &[impl AsRef<Path>]) -> Option<Box<dyn FontProvider>> {
    build_font_provider_for_families(extra_dirs, &[])
}

/// Build a font provider, loading only fonts whose family name matches one in `families`.
/// If `families` is empty, loads all available fonts (use with caution on systems with many fonts).
pub fn build_font_provider_for_families(
    extra_dirs: &[impl AsRef<Path>],
    families: &[&str],
) -> Option<Box<dyn FontProvider>> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    for dir in extra_dirs {
        db.load_fonts_dir(dir);
    }

    db.faces().next()?;

    let mut provider = BytesFontProvider::new();
    let mut loaded_paths: std::collections::HashSet<(String, u32)> =
        std::collections::HashSet::new();

    for face in db.faces() {
        let fontdb::Source::File(ref path) = face.source else {
            continue;
        };

        // Skip if no family matches (when filtering is active)
        if !families.is_empty() {
            let matches = face
                .families
                .iter()
                .any(|(name, _)| families.iter().any(|f| name.eq_ignore_ascii_case(f)));
            if !matches {
                // Log mono font misses for debugging
                if face
                    .families
                    .iter()
                    .any(|(n, _)| n.contains("Mono") || n.contains("mono"))
                {
                    tracing::debug!(
                        families = ?face.families.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
                        "Skipping mono font — no family match"
                    );
                }
                continue;
            }
        }

        let path_str = path.to_string_lossy().to_string();
        let key = (path_str.clone(), face.index);
        if loaded_paths.contains(&key) {
            continue;
        }

        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        let shared: Arc<[u8]> = Arc::from(bytes.as_slice());
        loaded_paths.insert(key);

        for family in &face.families {
            let style = match face.style {
                fontdb::Style::Normal => FontStyle::Normal,
                fontdb::Style::Italic => FontStyle::Italic,
                fontdb::Style::Oblique => FontStyle::Oblique,
            };

            provider.add_font_face(
                family.0.clone(),
                face.weight.0,
                style,
                shared.clone(),
                face.index,
            );
        }
    }

    if provider.available_families().is_empty() {
        return None;
    }

    Some(Box::new(provider))
}
