use crate::config::{HeaderLink, OxidocConfig, SocialConfig};

/// Build the header navigation links HTML.
pub(crate) fn build_header_nav(links: &[HeaderLink]) -> String {
    if links.is_empty() {
        return String::new();
    }
    let mut html = String::from(r#"<nav class="oxidoc-header-nav">"#);
    for link in links {
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<a href="{}" class="oxidoc-header-nav-link">{}</a>"#,
            crate::utils::html_escape(&link.href),
            crate::utils::html_escape(&link.label),
        );
    }
    html.push_str("</nav>");
    html
}

/// Build the mobile menu links (header links rendered as a list for mobile sidebar).
pub(crate) fn build_mobile_nav_links(links: &[HeaderLink]) -> String {
    if links.is_empty() {
        return String::new();
    }
    let mut html =
        String::from(r#"<nav class="oxidoc-mobile-nav-links" aria-label="Main navigation"><ul>"#);
    for link in links {
        use std::fmt::Write;
        let _ = write!(
            html,
            r#"<li><a href="{}">{}</a></li>"#,
            crate::utils::html_escape(&link.href),
            crate::utils::html_escape(&link.label),
        );
    }
    html.push_str("</ul></nav>");
    html
}

/// Build the header actions HTML with social links and icons injected.
pub(crate) fn build_header_actions(social: &SocialConfig) -> String {
    use crate::icons;
    use crate::template::HEADER_ACTIONS_HTML;

    let social_html = social.render_header_icons();
    let theme_icon = icons::svg_icon(icons::CONTRAST, "20", "20", "");
    let search_icon = icons::svg_icon(icons::SEARCH, "16", "16", "");

    HEADER_ACTIONS_HTML
        .replacen(
            r#"<div class="oxidoc-header-actions">"#,
            &format!(r#"<div class="oxidoc-header-actions">{social_html}"#),
            1,
        )
        .replacen(
            r#"class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme"></button>"#,
            &format!(r#"class="oxidoc-theme-toggle" aria-label="Toggle dark mode" title="Toggle theme">{theme_icon}</button>"#),
            1,
        )
        .replacen(
            r#"<span>Search</span>"#,
            &format!(r#"{search_icon}<span>Search</span>"#),
            1,
        )
}

/// Build the mobile menu toggle button (placed before logo in header).
pub(crate) fn build_menu_toggle() -> String {
    use crate::icons;

    let menu_icon = icons::svg_icon(icons::MENU, "24", "24", "oxidoc-menu-icon");
    let close_icon = icons::svg_icon(icons::CLOSE, "24", "24", "oxidoc-close-icon");

    format!(
        r#"<button class="oxidoc-menu-toggle" aria-label="Open navigation menu" aria-expanded="false">{menu_icon}{close_icon}</button>"#
    )
}

/// Generate the logo HTML for the header.
pub(crate) fn render_logo_html(config: &OxidocConfig) -> (String, String) {
    let safe_name = crate::utils::html_escape(&config.project.name);
    let logo_html = if let Some(ref logo) = config.project.logo {
        let safe_logo = crate::utils::html_escape(logo);
        format!(
            r#"<a href="/" class="oxidoc-logo"><img src="{safe_logo}" alt="{safe_name}" class="oxidoc-logo-img"> <span>{safe_name}</span></a>"#
        )
    } else {
        format!(r#"<a href="/" class="oxidoc-logo">{safe_name}</a>"#)
    };
    (logo_html, safe_name)
}
