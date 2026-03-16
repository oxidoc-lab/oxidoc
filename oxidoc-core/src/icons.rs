// Inline SVG icons — loaded at compile time, no CDN dependency.

// Social icons
pub const GITHUB: &str = include_str!("icons/github.svg");
pub const DISCORD: &str = include_str!("icons/discord.svg");
pub const TWITTER: &str = include_str!("icons/twitter.svg");
pub const MASTODON: &str = include_str!("icons/mastodon.svg");

// UI icons
pub const LIGHT_MODE: &str = include_str!("icons/light-mode.svg");
pub const DARK_MODE: &str = include_str!("icons/dark-mode.svg");
pub const CONTRAST: &str = include_str!("icons/contrast.svg");
pub const SEARCH: &str = include_str!("icons/search.svg");
pub const CLOSE: &str = include_str!("icons/close.svg");
pub const ARROW_UP: &str = include_str!("icons/arrow-up.svg");
pub const ARROW_BACK: &str = include_str!("icons/arrow-back.svg");
pub const AUTO_AWESOME: &str = include_str!("icons/auto-awesome.svg");
pub const TAG: &str = include_str!("icons/tag.svg");
pub const DESCRIPTION: &str = include_str!("icons/description.svg");
pub const LINK: &str = include_str!("icons/link.svg");
pub const MENU: &str = include_str!("icons/menu.svg");

/// Wrap an SVG string with a given width/height and optional class.
pub fn svg_icon(svg: &str, width: &str, height: &str, class: &str) -> String {
    // Replace width="1em" height="1em" with the desired size
    let mut s = svg
        .replace(r#"width="1em""#, &format!(r#"width="{width}""#))
        .replace(r#"height="1em""#, &format!(r#"height="{height}""#));
    if !class.is_empty() {
        s = s.replace("<svg ", &format!(r#"<svg class="{class}" "#));
    }
    s
}
