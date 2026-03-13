/// Optional asset paths and SRI hashes for CSS/JS resources.
#[derive(Debug, Default, Clone)]
pub struct AssetConfig<'a> {
    pub css_path: Option<&'a str>,
    pub js_path: Option<&'a str>,
    pub css_sri: Option<&'a str>,
    pub js_sri: Option<&'a str>,
}

/// Build a `<link rel="stylesheet">` tag with optional SRI integrity hash.
pub(crate) fn build_stylesheet_link(href: &str, sri: Option<&str>) -> String {
    if let Some(sri) = sri {
        format!(
            r#"    <link rel="stylesheet" href="{href}" integrity="{sri}" crossorigin="anonymous">"#
        )
    } else {
        format!(r#"    <link rel="stylesheet" href="{href}">"#)
    }
}

/// Build a `<script>` tag with optional SRI integrity hash.
pub(crate) fn build_script_tag(src: &str, sri: Option<&str>) -> String {
    if let Some(sri) = sri {
        format!(
            r#"    <script src="{src}" type="module" async integrity="{sri}" crossorigin="anonymous"></script>"#
        )
    } else {
        format!(r#"    <script src="{src}" type="module" async></script>"#)
    }
}

/// Build `<link rel="preload">` tags for CSS and JS assets with optional SRI.
pub(crate) fn build_preload_links(
    css_href: &str,
    css_sri: Option<&str>,
    js_src: &str,
    js_sri: Option<&str>,
) -> (String, String) {
    let css_preload = if let Some(sri) = css_sri {
        format!(
            r#"    <link rel="preload" href="{css_href}" as="style" integrity="{sri}" crossorigin="anonymous">"#
        )
    } else {
        format!(r#"    <link rel="preload" href="{css_href}" as="style">"#)
    };
    let js_preload = if let Some(sri) = js_sri {
        format!(
            r#"    <link rel="preload" href="{js_src}" as="script" integrity="{sri}" crossorigin="anonymous">"#
        )
    } else {
        format!(r#"    <link rel="preload" href="{js_src}" as="script">"#)
    };
    (css_preload, js_preload)
}
