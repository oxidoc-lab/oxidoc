use crate::crawler::NavGroup;
use crate::error::{OxidocError, Result};
use crate::utils::{extract_plain_text, xml_escape};
use rdx_parser::parse;
use std::path::Path;

/// Generate an Atom/RSS feed (feed.xml) from documentation pages.
pub fn generate_feed(
    nav_groups: &[NavGroup],
    project_name: &str,
    base_url: &str,
    description: &str,
    output_dir: &Path,
) -> Result<()> {
    let now = chrono::Utc::now();
    let feed_id = format!(
        "urn:oxidoc:{}",
        project_name.to_lowercase().replace(' ', "-")
    );
    let feed_link = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{}/", base_url)
    };

    let mut entries = String::new();
    for group in nav_groups {
        for page in &group.pages {
            let content =
                std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                    path: page.file_path.display().to_string(),
                    source: e,
                })?;

            let root = parse(&content);
            let summary =
                extract_first_paragraph(&root).unwrap_or_else(|| "(No summary)".to_string());

            let page_url = format!("{}{}.html", feed_link, page.slug);
            let entry_id = format!("urn:oxidoc:page:{}", page.slug);
            let updated = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

            entries.push_str(&format!(
                r#"  <entry>
    <id>{}</id>
    <title>{}</title>
    <link href="{}"/>
    <updated>{}</updated>
    <summary type="text">{}</summary>
  </entry>
"#,
                xml_escape(&entry_id),
                xml_escape(&page.title),
                xml_escape(&page_url),
                updated,
                xml_escape(&summary),
            ));
        }
    }

    let updated = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let atom_xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <id>{}</id>
  <title>{}</title>
  <link href="{}"/>
  <updated>{}</updated>
  <author>
    <name>{}</name>
  </author>
  <summary>{}</summary>
{}
</feed>
"#,
        xml_escape(&feed_id),
        xml_escape(project_name),
        xml_escape(&feed_link),
        updated,
        xml_escape(project_name),
        xml_escape(description),
        entries,
    );

    std::fs::write(output_dir.join("feed.xml"), atom_xml).map_err(|e| OxidocError::FileWrite {
        path: output_dir.join("feed.xml").display().to_string(),
        source: e,
    })?;

    Ok(())
}

/// Extract the first paragraph as a summary, truncated to 160 chars.
fn extract_first_paragraph(root: &rdx_ast::Root) -> Option<String> {
    for node in &root.children {
        if matches!(node, rdx_ast::Node::Paragraph(_)) {
            let text = extract_plain_text(node);
            if !text.trim().is_empty() {
                return Some(text.chars().take(160).collect());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feed_generation_writes_file() {
        let tmp = tempfile::tempdir().unwrap();
        let nav_groups = vec![];
        let result = generate_feed(
            &nav_groups,
            "Test Project",
            "https://example.com",
            "A test project",
            tmp.path(),
        );
        assert!(result.is_ok());
        assert!(tmp.path().join("feed.xml").exists());

        let content = std::fs::read_to_string(tmp.path().join("feed.xml")).unwrap();
        assert!(content.contains(r#"xmlns="http://www.w3.org/2005/Atom""#));
        assert!(content.contains("Test Project"));
    }
}
