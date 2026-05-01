use std::fmt::Write;

use super::config::ResolvedLlm;
use super::emit::top_level_segment;
use crate::utils::html_escape;

/// Build the "Copy Markdown" dropdown HTML for a single page.
///
/// `raw_markdown` is the page's source content; it is embedded inline in a
/// hidden `<script type="text/markdown">` block so the Copy button can read it
/// directly without an extra HTTP fetch.
///
/// Returns an empty string when the button is disabled for this page.
pub fn render_copy_markdown_button(
    slug: &str,
    raw_markdown: &str,
    resolved: &ResolvedLlm,
) -> String {
    if !resolved.enabled || !resolved.copy_button {
        return String::new();
    }

    let mut menu = String::new();

    if let Some(section) = top_level_segment(slug) {
        let section_full = format!("/{section}/llms-full.txt");
        let _ = write!(
            menu,
            r#"<li><a class="oxidoc-llm-menu-item" href="{href}" target="_blank" rel="noopener"><span class="oxidoc-llm-menu-title">View /{section}/llms-full.txt</span><span class="oxidoc-llm-menu-sub">Full text of the {section} section</span></a></li>"#,
            href = html_escape(&section_full),
            section = html_escape(section),
        );
    }

    let _ = write!(
        menu,
        r#"<li><a class="oxidoc-llm-menu-item" href="/llms-full.txt" target="_blank" rel="noopener"><span class="oxidoc-llm-menu-title">View /llms-full.txt</span><span class="oxidoc-llm-menu-sub">Full documentation for AI</span></a></li>"#,
    );
    let _ = write!(
        menu,
        r#"<li><a class="oxidoc-llm-menu-item" href="/llms.txt" target="_blank" rel="noopener"><span class="oxidoc-llm-menu-title">View /llms.txt</span><span class="oxidoc-llm-menu-sub">Site index for AI</span></a></li>"#,
    );

    let safe_markdown = escape_for_script_block(raw_markdown);

    format!(
        r#"<div class="oxidoc-llm-actions">
  <script type="text/markdown" class="oxidoc-llm-source">{safe_markdown}</script>
  <button type="button" class="oxidoc-llm-copy" aria-label="Copy this page as Markdown">
    <span class="oxidoc-llm-copy-label">Copy Markdown</span>
  </button>
  <button type="button" class="oxidoc-llm-toggle" aria-haspopup="true" aria-expanded="false" aria-label="More options">
    <svg aria-hidden="true" width="14" height="14" viewBox="0 0 24 24"><path fill="currentColor" d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6l-6-6z"/></svg>
  </button>
  <ul class="oxidoc-llm-menu" role="menu" hidden>{menu}</ul>
</div>"#,
    )
}

/// Make a string safe to embed as the body of `<script type="text/markdown">`.
///
/// Inside a `<script>` element only the substring `</script` (case-insensitive,
/// with a trailing whitespace or `>`) terminates the block. Splitting that
/// pattern with a backslash is enough.
fn escape_for_script_block(s: &str) -> String {
    s.replace("</script", "<\\/script")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled() -> ResolvedLlm {
        ResolvedLlm {
            enabled: true,
            copy_button: true,
        }
    }

    #[test]
    fn empty_when_disabled() {
        let mut r = enabled();
        r.enabled = false;
        assert!(render_copy_markdown_button("intro", "body", &r).is_empty());
    }

    #[test]
    fn empty_when_copy_button_off() {
        let mut r = enabled();
        r.copy_button = false;
        assert!(render_copy_markdown_button("intro", "body", &r).is_empty());
    }

    #[test]
    fn embeds_raw_markdown_inline() {
        let html = render_copy_markdown_button("intro", "# Hello\nworld", &enabled());
        assert!(html.contains(r#"<script type="text/markdown" class="oxidoc-llm-source"># Hello"#));
        assert!(html.contains("world</script>"));
    }

    #[test]
    fn escapes_script_terminator() {
        let html = render_copy_markdown_button("intro", "before </script> after", &enabled());
        assert!(!html.contains("before </script> after"));
        assert!(html.contains(r"before <\/script> after"));
    }

    #[test]
    fn includes_root_links() {
        let html = render_copy_markdown_button("intro", "x", &enabled());
        assert!(html.contains(r#"href="/llms-full.txt""#));
        assert!(html.contains(r#"href="/llms.txt""#));
    }

    #[test]
    fn includes_section_link() {
        let html = render_copy_markdown_button("docs/intro", "x", &enabled());
        assert!(html.contains("/docs/llms-full.txt"));
    }
}
