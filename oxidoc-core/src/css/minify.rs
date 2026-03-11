use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

/// Minify CSS using lightningcss.
///
/// If parsing or minification fails, the original CSS is returned unmodified.
pub fn minify_css(css: &str) -> String {
    let mut stylesheet = match StyleSheet::parse(css, ParserOptions::default()) {
        Ok(s) => s,
        Err(_) => return css.to_string(),
    };

    if stylesheet.minify(MinifyOptions::default()).is_err() {
        return css.to_string();
    }

    match stylesheet.to_css(PrinterOptions {
        minify: true,
        ..Default::default()
    }) {
        Ok(result) => result.code,
        Err(_) => css.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_reduces_size() {
        let css = "body { color: red; margin: 0; padding: 0; }";
        let minified = minify_css(css);
        assert!(minified.len() <= css.len());
        assert!(minified.contains("color"));
    }

    #[test]
    fn minify_handles_invalid_css() {
        let invalid = "this is {{ not valid css {{{{";
        let result = minify_css(invalid);
        assert_eq!(result, invalid);
    }
}
