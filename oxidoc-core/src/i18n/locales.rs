//! Locale display name lookup table.

/// Get the display name for a locale code.
pub fn locale_display_name(locale: &str) -> &str {
    match locale {
        "en" => "English",
        "es" => "Español",
        "ja" => "日本語",
        "fr" => "Français",
        "de" => "Deutsch",
        "zh" => "中文",
        "ko" => "한국어",
        "pt" => "Português",
        "ru" => "Русский",
        "it" => "Italiano",
        "ar" => "العربية",
        "hi" => "हिन्दी",
        "tr" => "Türkçe",
        "pl" => "Polski",
        "vi" => "Tiếng Việt",
        "th" => "ไทย",
        "nl" => "Nederlands",
        "sv" => "Svenska",
        "da" => "Dansk",
        "fi" => "Suomi",
        "no" => "Norsk",
        "cs" => "Čeština",
        "hu" => "Magyar",
        "ro" => "Română",
        "el" => "Ελληνικά",
        "he" => "עברית",
        "uk" => "Українська",
        _ => locale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_display_name() {
        assert_eq!(locale_display_name("en"), "English");
        assert_eq!(locale_display_name("es"), "Español");
        assert_eq!(locale_display_name("ja"), "日本語");
        assert_eq!(locale_display_name("fr"), "Français");
    }

    #[test]
    fn test_locale_display_name_unknown() {
        assert_eq!(locale_display_name("unknown"), "unknown");
    }
}
