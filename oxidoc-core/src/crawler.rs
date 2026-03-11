use crate::config::{OxidocConfig, RoutingConfig};
use crate::error::{OxidocError, Result};
use std::path::{Path, PathBuf};

/// A resolved page ready for rendering.
#[derive(Debug, Clone)]
pub struct PageEntry {
    /// Display title derived from filename or frontmatter.
    pub title: String,
    /// URL slug (e.g., "intro", "getting-started/quickstart").
    pub slug: String,
    /// Absolute path to the `.rdx` file.
    pub file_path: PathBuf,
    /// Navigation group this page belongs to.
    pub group: Option<String>,
}

/// A resolved navigation group for sidebar rendering.
#[derive(Debug, Clone)]
pub struct NavGroup {
    pub title: String,
    pub pages: Vec<PageEntry>,
}

/// Discover all pages to render, using either explicit routing or file-system fallback.
pub fn discover_pages(project_root: &Path, config: &OxidocConfig) -> Result<Vec<NavGroup>> {
    if config.routing.navigation.is_empty() {
        discover_filesystem(project_root)
    } else {
        discover_explicit(project_root, &config.routing)
    }
}

/// File-system fallback: traverse `docs/` directory, ordering by numeric prefix or alphabetically.
fn discover_filesystem(project_root: &Path) -> Result<Vec<NavGroup>> {
    let docs_dir = project_root.join("docs");
    if !docs_dir.is_dir() {
        return Err(OxidocError::DocsNotFound {
            path: docs_dir.display().to_string(),
        });
    }

    let mut entries = collect_rdx_files(&docs_dir, &docs_dir)?;
    entries.sort_by(|a, b| a.slug.cmp(&b.slug));

    Ok(vec![NavGroup {
        title: String::new(),
        pages: entries,
    }])
}

/// Explicit routing: resolve slugs from `[routing.navigation]` to `.rdx` files.
fn discover_explicit(project_root: &Path, routing: &RoutingConfig) -> Result<Vec<NavGroup>> {
    let docs_dir = project_root.join("docs");
    let mut groups = Vec::new();

    for nav in &routing.navigation {
        let mut pages = Vec::new();
        for slug in &nav.pages {
            let file_path = docs_dir.join(format!("{slug}.rdx"));
            if !file_path.is_file() {
                return Err(OxidocError::PageNotFound { slug: slug.clone() });
            }
            pages.push(PageEntry {
                title: slug_to_title(slug),
                slug: slug.clone(),
                file_path,
                group: Some(nav.group.clone()),
            });
        }
        groups.push(NavGroup {
            title: nav.group.clone(),
            pages,
        });
    }

    Ok(groups)
}

/// Recursively collect `.rdx` files from a directory.
fn collect_rdx_files(dir: &Path, docs_root: &Path) -> Result<Vec<PageEntry>> {
    let mut entries = Vec::new();
    let read_dir = std::fs::read_dir(dir).map_err(|e| OxidocError::FileRead {
        path: dir.display().to_string(),
        source: e,
    })?;

    for entry in read_dir {
        let entry = entry.map_err(|e| OxidocError::FileRead {
            path: dir.display().to_string(),
            source: e,
        })?;
        let path = entry.path();

        if path.is_dir() {
            entries.extend(collect_rdx_files(&path, docs_root)?);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rdx") {
            let relative =
                path.strip_prefix(docs_root)
                    .map_err(|_| OxidocError::PathNotUnderRoot {
                        path: path.display().to_string(),
                        root: docs_root.display().to_string(),
                    })?;
            let slug = relative
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            entries.push(PageEntry {
                title: slug_to_title(&slug),
                slug,
                file_path: path,
                group: None,
            });
        }
    }

    Ok(entries)
}

/// Convert a slug like "01-getting-started" into "Getting Started".
fn slug_to_title(slug: &str) -> String {
    let basename = slug.rsplit('/').next().unwrap_or(slug);
    // Strip leading numeric prefix (e.g., "01-" or "1-")
    let stripped = basename
        .find('-')
        .and_then(|pos| {
            if basename[..pos].chars().all(|c| c.is_ascii_digit()) {
                Some(&basename[pos + 1..])
            } else {
                None
            }
        })
        .unwrap_or(basename);

    stripped
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    format!("{upper}{}", chars.as_str())
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_to_title_basic() {
        assert_eq!(slug_to_title("intro"), "Intro");
        assert_eq!(slug_to_title("getting-started"), "Getting Started");
        assert_eq!(slug_to_title("01-intro"), "Intro");
        assert_eq!(slug_to_title("02-getting-started"), "Getting Started");
    }

    #[test]
    fn slug_to_title_nested() {
        assert_eq!(slug_to_title("guides/01-quickstart"), "Quickstart");
    }

    #[test]
    fn slug_to_title_no_prefix() {
        assert_eq!(slug_to_title("my-page"), "My Page");
    }

    #[test]
    fn discover_filesystem_missing_docs() {
        let tmp = tempfile::tempdir().unwrap();
        let err = discover_filesystem(tmp.path()).unwrap_err();
        assert!(matches!(err, OxidocError::DocsNotFound { .. }));
    }

    #[test]
    fn discover_filesystem_collects_rdx() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir(&docs).unwrap();
        std::fs::write(docs.join("01-intro.rdx"), "# Intro").unwrap();
        std::fs::write(docs.join("02-setup.rdx"), "# Setup").unwrap();
        std::fs::write(docs.join("readme.md"), "ignored").unwrap();

        let groups = discover_filesystem(tmp.path()).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].pages.len(), 2);
        assert_eq!(groups[0].pages[0].slug, "01-intro");
        assert_eq!(groups[0].pages[0].title, "Intro");
        assert_eq!(groups[0].pages[1].slug, "02-setup");
    }

    #[test]
    fn discover_filesystem_nested_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let docs = tmp.path().join("docs");
        let guides = docs.join("guides");
        std::fs::create_dir_all(&guides).unwrap();
        std::fs::write(docs.join("intro.rdx"), "").unwrap();
        std::fs::write(guides.join("setup.rdx"), "").unwrap();

        let groups = discover_filesystem(tmp.path()).unwrap();
        let slugs: Vec<&str> = groups[0].pages.iter().map(|p| p.slug.as_str()).collect();
        assert!(slugs.contains(&"guides/setup"));
        assert!(slugs.contains(&"intro"));
    }
}
