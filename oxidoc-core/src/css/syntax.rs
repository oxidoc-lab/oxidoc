/// Syntax highlighting token styles for code blocks.
/// Supports both light and dark color schemes via CSS variables.
pub const SYNTAX: &str = include_str!("styles/syntax.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_contains_all_token_classes() {
        assert!(SYNTAX.contains(".tok-keyword"));
        assert!(SYNTAX.contains(".tok-string"));
        assert!(SYNTAX.contains(".tok-comment"));
        assert!(SYNTAX.contains(".tok-number"));
        assert!(SYNTAX.contains(".tok-function"));
        assert!(SYNTAX.contains(".tok-type"));
        assert!(SYNTAX.contains(".tok-operator"));
        assert!(SYNTAX.contains(".tok-punctuation"));
        assert!(SYNTAX.contains(".tok-property"));
        assert!(SYNTAX.contains(".tok-variable"));
        assert!(SYNTAX.contains(".tok-builtin"));
        assert!(SYNTAX.contains(".tok-attr"));
    }

    #[test]
    fn syntax_has_light_theme() {
        assert!(SYNTAX.contains("prefers-color-scheme: light"));
    }

    #[test]
    fn syntax_has_dark_theme() {
        assert!(SYNTAX.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn syntax_has_manual_theme_override() {
        assert!(SYNTAX.contains("html[data-theme=\"light\"]"));
        assert!(SYNTAX.contains("html[data-theme=\"dark\"]"));
    }
}
