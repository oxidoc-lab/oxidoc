/// Inject the version switcher HTML into a rendered page.
/// Inserts before the header actions div (social icons, theme toggle, search).
pub(crate) fn inject_version_switcher(html: &str, switcher_html: &str) -> String {
    if switcher_html.is_empty() {
        return html.to_string();
    }
    if html.contains(r#"<div class="oxidoc-header-actions">"#) {
        html.replacen(
            r#"<div class="oxidoc-header-actions">"#,
            &format!("<div class=\"oxidoc-header-actions\">{switcher_html}"),
            1,
        )
    } else {
        html.replacen("</header>", &format!("{switcher_html}</header>"), 1)
    }
}

/// Inject the "Copy Markdown / Open in LLM" dropdown into a rendered page.
/// Replaces the `<span class="oxidoc-llm-slot" hidden></span>` placeholder
/// emitted by the page template. Pages where the button is disabled keep the
/// (invisible) placeholder, so unrelated render paths (archive, API, root)
/// remain unaffected.
pub(crate) fn inject_copy_markdown_button(html: &str, button_html: &str) -> String {
    if button_html.is_empty() {
        return html.to_string();
    }
    html.replacen(
        r#"<span class="oxidoc-llm-slot" hidden></span>"#,
        button_html,
        1,
    )
}

/// Inject an outdated version banner and noindex meta tag into archived pages.
pub(crate) fn inject_outdated_banner(html: &str, version: &str) -> String {
    let banner = format!(
        "<div class=\"oxidoc-outdated-banner\" role=\"alert\">You are viewing docs for <strong>{version}</strong>. <a href=\"/\" onclick=\"var p=location.pathname.replace(/^\\/+|\\/$/g,'').split('/');if(p[0]==='{version}')p.shift();this.href='/'+(p.length>0?p[0]+'/':'');return true;\">Switch to latest</a>.</div>"
    );
    let html = html.replacen(
        "</head>",
        r#"    <meta name="robots" content="noindex, nofollow">
</head>"#,
        1,
    );
    // Insert before the main layout div (works for both regular and landing pages)
    if html.contains("<div class=\"oxidoc-layout") {
        html.replacen(
            "<div class=\"oxidoc-layout",
            &format!("{banner}<div class=\"oxidoc-layout"),
            1,
        )
    } else {
        html.replacen("<main ", &format!("{banner}<main "), 1)
    }
}
