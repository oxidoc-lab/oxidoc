use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::config::{LlmConfig, ResolvedLlm};
use crate::error::{OxidocError, Result};

/// Per-page input for LLM output generation.
pub struct PageLlmInput {
    pub slug: String,
    pub title: String,
    pub file_path: PathBuf,
    pub resolved: ResolvedLlm,
}

/// Generate all LLM output files for a built site.
///
/// Emits, conditional on resolved settings:
/// - Root `llms.txt` + `llms-full.txt`
/// - Per-section `<section>/llms.txt` + `<section>/llms-full.txt`
///   (top-level URL segment groups)
///
/// Pages whose resolved `enabled` is false are omitted from every output.
pub fn generate_llm_outputs(
    pages: &[PageLlmInput],
    config: &LlmConfig,
    output_dir: &Path,
) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let mut contents: Vec<(String, String)> = Vec::with_capacity(pages.len());
    for page in pages {
        if !page.resolved.enabled {
            continue;
        }
        let content =
            std::fs::read_to_string(&page.file_path).map_err(|e| OxidocError::FileRead {
                path: page.file_path.display().to_string(),
                source: e,
            })?;
        contents.push((page.slug.clone(), content));
    }

    let root_summary = build_summary(pages);
    let root_full = build_full(pages, &contents);
    write_with_parents(&output_dir.join("llms.txt"), &root_summary)?;
    write_with_parents(&output_dir.join("llms-full.txt"), &root_full)?;

    if config.section_files {
        let mut sections: BTreeMap<&str, Vec<&PageLlmInput>> = BTreeMap::new();
        for page in pages {
            if !page.resolved.enabled {
                continue;
            }
            if let Some(section) = top_level_segment(&page.slug) {
                sections.entry(section).or_default().push(page);
            }
        }

        for (section, section_pages) in sections {
            let summary = build_summary_for(section_pages.iter().copied());
            let full = build_full_for(section_pages.iter().copied(), &contents);
            let dir = output_dir.join(section);
            write_with_parents(&dir.join("llms.txt"), &summary)?;
            write_with_parents(&dir.join("llms-full.txt"), &full)?;
        }
    }

    Ok(())
}

/// Top-level URL segment of a slug (`"docs/api/auth"` → `Some("docs")`).
/// Returns `None` for root-level pages with no slash.
pub fn top_level_segment(slug: &str) -> Option<&str> {
    slug.split_once('/').map(|(head, _)| head)
}

fn lookup<'a>(contents: &'a [(String, String)], slug: &str) -> Option<&'a str> {
    contents
        .iter()
        .find(|(s, _)| s == slug)
        .map(|(_, c)| c.as_str())
}

fn build_summary(pages: &[PageLlmInput]) -> String {
    build_summary_for(pages.iter().filter(|p| p.resolved.enabled))
}

fn build_summary_for<'a, I: IntoIterator<Item = &'a PageLlmInput>>(pages: I) -> String {
    let mut out = String::new();
    for page in pages {
        out.push_str(&format!("- /{}: {}\n", page.slug, page.title));
    }
    out
}

fn build_full(pages: &[PageLlmInput], contents: &[(String, String)]) -> String {
    build_full_for(pages.iter().filter(|p| p.resolved.enabled), contents)
}

fn build_full_for<'a, I: IntoIterator<Item = &'a PageLlmInput>>(
    pages: I,
    contents: &[(String, String)],
) -> String {
    let mut out = String::new();
    for page in pages {
        let body = lookup(contents, &page.slug).unwrap_or("");
        out.push_str(&format!(
            "\n---\n# {} ({})\n\n{}\n",
            page.title, page.slug, body
        ));
    }
    out
}

fn write_with_parents(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| OxidocError::DirCreate {
            path: parent.display().to_string(),
            source: e,
        })?;
    }
    std::fs::write(path, content).map_err(|e| OxidocError::FileWrite {
        path: path.display().to_string(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(slug: &str, title: &str, file: &Path, enabled: bool) -> PageLlmInput {
        PageLlmInput {
            slug: slug.into(),
            title: title.into(),
            file_path: file.to_path_buf(),
            resolved: ResolvedLlm {
                enabled,
                copy_button: true,
            },
        }
    }

    #[test]
    fn top_level_segment_basic() {
        assert_eq!(top_level_segment("docs/intro"), Some("docs"));
        assert_eq!(top_level_segment("docs/api/auth"), Some("docs"));
        assert_eq!(top_level_segment("intro"), None);
    }

    #[test]
    fn emits_root_and_section_files() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir(&src).unwrap();
        let f1 = src.join("a.rdx");
        let f2 = src.join("b.rdx");
        let f3 = src.join("c.rdx");
        std::fs::write(&f1, "# A\nbody-a").unwrap();
        std::fs::write(&f2, "# B\nbody-b").unwrap();
        std::fs::write(&f3, "# C\nbody-c").unwrap();

        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();

        let pages = vec![
            page("docs/a", "A", &f1, true),
            page("docs/b", "B", &f2, true),
            page("guides/c", "C", &f3, true),
        ];
        generate_llm_outputs(&pages, &LlmConfig::default(), &out).unwrap();

        let root_summary = std::fs::read_to_string(out.join("llms.txt")).unwrap();
        assert!(root_summary.contains("/docs/a") && root_summary.contains("/guides/c"));
        let root_full = std::fs::read_to_string(out.join("llms-full.txt")).unwrap();
        assert!(root_full.contains("body-a") && root_full.contains("body-c"));

        let docs_summary = std::fs::read_to_string(out.join("docs/llms.txt")).unwrap();
        assert!(docs_summary.contains("/docs/a"));
        assert!(!docs_summary.contains("/guides/c"));
        let docs_full = std::fs::read_to_string(out.join("docs/llms-full.txt")).unwrap();
        assert!(docs_full.contains("body-a") && docs_full.contains("body-b"));
        assert!(!docs_full.contains("body-c"));

        let guides_full = std::fs::read_to_string(out.join("guides/llms-full.txt")).unwrap();
        assert!(guides_full.contains("body-c") && !guides_full.contains("body-a"));
    }

    #[test]
    fn disabled_page_omitted_everywhere() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir(&src).unwrap();
        let f1 = src.join("a.rdx");
        let f2 = src.join("b.rdx");
        std::fs::write(&f1, "body-a").unwrap();
        std::fs::write(&f2, "body-b").unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();

        let pages = vec![
            page("docs/a", "A", &f1, true),
            page("docs/secret", "S", &f2, false),
        ];
        generate_llm_outputs(&pages, &LlmConfig::default(), &out).unwrap();

        let summary = std::fs::read_to_string(out.join("llms.txt")).unwrap();
        assert!(summary.contains("/docs/a"));
        assert!(!summary.contains("/docs/secret"));
    }

    #[test]
    fn section_files_disabled() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        std::fs::create_dir(&src).unwrap();
        let f1 = src.join("a.rdx");
        std::fs::write(&f1, "body").unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();

        let cfg = LlmConfig {
            section_files: false,
            ..LlmConfig::default()
        };
        let pages = vec![page("docs/a", "A", &f1, true)];
        generate_llm_outputs(&pages, &cfg, &out).unwrap();

        assert!(out.join("llms.txt").exists());
        assert!(!out.join("docs/llms.txt").exists());
    }

    #[test]
    fn site_disabled_emits_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let cfg = LlmConfig {
            enabled: false,
            ..LlmConfig::default()
        };
        generate_llm_outputs(&[], &cfg, &out).unwrap();
        assert!(!out.join("llms.txt").exists());
    }
}
