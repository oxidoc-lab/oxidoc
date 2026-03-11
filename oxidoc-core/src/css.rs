use crate::config::OxidocConfig;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};

/// Generate the base CSS for an Oxidoc site.
pub fn generate_base_css(config: &OxidocConfig) -> String {
    let primary = &config.theme.primary;
    let dark_mode = &config.theme.dark_mode;

    let dark_scheme_rule = match dark_mode.as_str() {
        "dark" => r#"
html { color-scheme: dark; }
:root {
    --oxidoc-bg: #0f172a;
    --oxidoc-bg-secondary: #1e293b;
    --oxidoc-text: #e2e8f0;
    --oxidoc-text-secondary: #94a3b8;
    --oxidoc-border: #334155;
    --oxidoc-code-bg: #1e293b;
}
"#
        .to_string(),
        "light" => r#"
html { color-scheme: light; }
:root {
    --oxidoc-bg: #ffffff;
    --oxidoc-bg-secondary: #f8fafc;
    --oxidoc-text: #1e293b;
    --oxidoc-text-secondary: #64748b;
    --oxidoc-border: #e2e8f0;
    --oxidoc-code-bg: #f1f5f9;
}
"#
        .to_string(),
        _ => r#"
:root {
    --oxidoc-bg: #ffffff;
    --oxidoc-bg-secondary: #f8fafc;
    --oxidoc-text: #1e293b;
    --oxidoc-text-secondary: #64748b;
    --oxidoc-border: #e2e8f0;
    --oxidoc-code-bg: #f1f5f9;
}

@media (prefers-color-scheme: dark) {
    :root {
        --oxidoc-bg: #0f172a;
        --oxidoc-bg-secondary: #1e293b;
        --oxidoc-text: #e2e8f0;
        --oxidoc-text-secondary: #94a3b8;
        --oxidoc-border: #334155;
        --oxidoc-code-bg: #1e293b;
    }
}
"#
        .to_string(),
    };

    format!(
        r#"/* Oxidoc Base Stylesheet — generated */
:root {{
    --oxidoc-primary: {primary};
    --oxidoc-font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    --oxidoc-font-mono: "SF Mono", "Fira Code", "Fira Mono", "Roboto Mono", Menlo, Consolas, monospace;
    --oxidoc-content-max: 48rem;
    --oxidoc-sidebar-width: 16rem;
    --oxidoc-toc-width: 14rem;
    --oxidoc-header-height: 3.5rem;
}}

{dark_scheme_rule}

/* Reset */
*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
html {{ font-size: 16px; -webkit-text-size-adjust: 100%; }}
body {{
    font-family: var(--oxidoc-font-sans);
    color: var(--oxidoc-text);
    background: var(--oxidoc-bg);
    line-height: 1.7;
    min-height: 100vh;
}}

/* Header */
.oxidoc-header {{
    position: sticky;
    top: 0;
    z-index: 100;
    height: var(--oxidoc-header-height);
    display: flex;
    align-items: center;
    padding: 0 1.5rem;
    background: var(--oxidoc-bg);
    border-bottom: 1px solid var(--oxidoc-border);
}}
.oxidoc-logo {{
    font-weight: 700;
    font-size: 1.125rem;
    color: var(--oxidoc-text);
    text-decoration: none;
}}
.oxidoc-logo:hover {{ color: var(--oxidoc-primary); }}

/* Layout — 3-column */
.oxidoc-layout {{
    display: grid;
    grid-template-columns: var(--oxidoc-sidebar-width) minmax(0, 1fr) var(--oxidoc-toc-width);
    min-height: calc(100vh - var(--oxidoc-header-height));
}}

/* Sidebar */
.oxidoc-sidebar {{
    position: sticky;
    top: var(--oxidoc-header-height);
    height: calc(100vh - var(--oxidoc-header-height));
    overflow-y: auto;
    padding: 1.5rem 1rem;
    border-right: 1px solid var(--oxidoc-border);
    background: var(--oxidoc-bg-secondary);
    scrollbar-width: thin;
}}
.oxidoc-nav-title {{
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--oxidoc-text-secondary);
    margin-bottom: 0.5rem;
    margin-top: 1rem;
}}
.oxidoc-nav-group:first-child .oxidoc-nav-title {{ margin-top: 0; }}
.oxidoc-nav-group ul {{
    list-style: none;
    padding: 0;
}}
.oxidoc-nav-group li a {{
    display: block;
    padding: 0.25rem 0.75rem;
    border-radius: 0.375rem;
    color: var(--oxidoc-text-secondary);
    text-decoration: none;
    font-size: 0.875rem;
    transition: color 0.15s, background 0.15s;
}}
.oxidoc-nav-group li a:hover {{
    color: var(--oxidoc-text);
    background: var(--oxidoc-border);
}}
.oxidoc-nav-group li a.active {{
    color: var(--oxidoc-primary);
    background: color-mix(in srgb, var(--oxidoc-primary) 10%, transparent);
    font-weight: 500;
}}

/* Main content */
.oxidoc-content {{
    max-width: var(--oxidoc-content-max);
    margin: 0 auto;
    padding: 2rem 2.5rem;
    width: 100%;
}}

/* TOC sidebar */
.oxidoc-toc-sidebar {{
    position: sticky;
    top: var(--oxidoc-header-height);
    height: calc(100vh - var(--oxidoc-header-height));
    overflow-y: auto;
    padding: 1.5rem 1rem;
    border-left: 1px solid var(--oxidoc-border);
    scrollbar-width: thin;
}}
.oxidoc-toc {{
    font-size: 0.8125rem;
}}
.oxidoc-toc ul {{
    list-style: none;
    padding: 0;
}}
.oxidoc-toc li {{
    margin: 0.25rem 0;
}}
.oxidoc-toc li a {{
    color: var(--oxidoc-text-secondary);
    text-decoration: none;
    transition: color 0.15s;
}}
.oxidoc-toc li a:hover {{ color: var(--oxidoc-primary); }}
.oxidoc-toc .toc-level-3 {{ padding-left: 0.75rem; }}
.oxidoc-toc .toc-level-4 {{ padding-left: 1.5rem; }}

/* Breadcrumbs */
.oxidoc-breadcrumbs {{
    margin-bottom: 1.5rem;
    font-size: 0.8125rem;
}}
.oxidoc-breadcrumbs ol {{
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0;
    padding: 0;
}}
.oxidoc-breadcrumbs li {{ display: inline; }}
.oxidoc-breadcrumbs .separator {{
    margin: 0 0.375rem;
    color: var(--oxidoc-text-secondary);
}}
.oxidoc-breadcrumbs a {{
    color: var(--oxidoc-primary);
    text-decoration: none;
}}
.oxidoc-breadcrumbs a:hover {{ text-decoration: underline; }}
.oxidoc-breadcrumbs [aria-current="page"] {{
    color: var(--oxidoc-text-secondary);
}}

/* Typography */
article h1 {{ font-size: 2rem; font-weight: 700; margin-bottom: 1rem; line-height: 1.2; }}
article h2 {{
    font-size: 1.5rem;
    font-weight: 600;
    margin-top: 2.5rem;
    margin-bottom: 0.75rem;
    padding-bottom: 0.375rem;
    border-bottom: 1px solid var(--oxidoc-border);
    line-height: 1.3;
}}
article h3 {{ font-size: 1.25rem; font-weight: 600; margin-top: 2rem; margin-bottom: 0.5rem; }}
article h4 {{ font-size: 1.0625rem; font-weight: 600; margin-top: 1.5rem; margin-bottom: 0.5rem; }}
article h5, article h6 {{ font-size: 1rem; font-weight: 600; margin-top: 1.25rem; margin-bottom: 0.5rem; }}

article p {{ margin-bottom: 1rem; }}

article a {{
    color: var(--oxidoc-primary);
    text-decoration: underline;
    text-decoration-color: color-mix(in srgb, var(--oxidoc-primary) 40%, transparent);
    text-underline-offset: 2px;
    transition: text-decoration-color 0.15s;
}}
article a:hover {{ text-decoration-color: var(--oxidoc-primary); }}

article strong {{ font-weight: 600; }}
article em {{ font-style: italic; }}

article ul, article ol {{ padding-left: 1.5rem; margin-bottom: 1rem; }}
article li {{ margin-bottom: 0.25rem; }}
article li > p {{ margin-bottom: 0.5rem; }}

article blockquote {{
    border-left: 3px solid var(--oxidoc-primary);
    padding: 0.5rem 1rem;
    margin: 1rem 0;
    color: var(--oxidoc-text-secondary);
    background: var(--oxidoc-bg-secondary);
    border-radius: 0 0.375rem 0.375rem 0;
}}

article code {{
    font-family: var(--oxidoc-font-mono);
    font-size: 0.875em;
    background: var(--oxidoc-code-bg);
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
}}
article pre {{
    background: var(--oxidoc-code-bg);
    border: 1px solid var(--oxidoc-border);
    border-radius: 0.5rem;
    padding: 1rem;
    margin: 1rem 0;
    overflow-x: auto;
    font-size: 0.875rem;
    line-height: 1.6;
}}
article pre code {{
    background: none;
    padding: 0;
    border-radius: 0;
}}

article table {{
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
    font-size: 0.875rem;
}}
article th, article td {{
    padding: 0.625rem 0.75rem;
    border: 1px solid var(--oxidoc-border);
    text-align: left;
}}
article th {{
    background: var(--oxidoc-bg-secondary);
    font-weight: 600;
}}

article hr {{
    border: none;
    border-top: 1px solid var(--oxidoc-border);
    margin: 2rem 0;
}}

article img {{
    max-width: 100%;
    height: auto;
    border-radius: 0.5rem;
}}

/* Heading anchors */
article h1, article h2, article h3, article h4, article h5, article h6 {{
    scroll-margin-top: calc(var(--oxidoc-header-height) + 1rem);
}}

/* Responsive */
@media (max-width: 1024px) {{
    .oxidoc-layout {{
        grid-template-columns: 1fr;
    }}
    .oxidoc-sidebar, .oxidoc-toc-sidebar {{
        display: none;
    }}
    .oxidoc-content {{
        padding: 1.5rem 1rem;
    }}
}}
"#
    )
}

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
    use crate::config::parse_config;

    #[test]
    fn generates_base_css() {
        let config = parse_config("[project]\nname = \"Test\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("--oxidoc-primary: #3b82f6"));
        assert!(css.contains(".oxidoc-layout"));
        assert!(css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn generates_dark_mode_css() {
        let config =
            parse_config("[project]\nname = \"T\"\n[theme]\ndark_mode = \"dark\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("color-scheme: dark"));
        assert!(!css.contains("prefers-color-scheme"));
    }

    #[test]
    fn generates_light_mode_css() {
        let config =
            parse_config("[project]\nname = \"T\"\n[theme]\ndark_mode = \"light\"").unwrap();
        let css = generate_base_css(&config);
        assert!(css.contains("color-scheme: light"));
    }

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
