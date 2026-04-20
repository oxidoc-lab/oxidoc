//! Integration tests for the version archive system.
//!
//! Tests the full archive → build pipeline: creating archives, building sites
//! with archived versions, version switcher injection, and multi-version builds.

use oxidoc_core::archive::{create_archive, write_archive};
use oxidoc_core::builder::build_site;

/// Helper to create a temporary test project with config and doc files.
fn setup_project(
    config_toml: &str,
    files: &[(&str, &str)],
) -> (tempfile::TempDir, std::path::PathBuf) {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let root = tmp.path();
    let docs = root.join("docs");
    std::fs::create_dir(&docs).expect("Failed to create docs dir");
    std::fs::write(root.join("oxidoc.toml"), config_toml).expect("Failed to write config");
    for (name, content) in files {
        // Support nested paths (e.g., "sub/page.rdx")
        let path = docs.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent dir");
        }
        std::fs::write(&path, content).expect("Failed to write file");
    }
    let output = root.join("dist");
    (tmp, output)
}

fn read_output(output: &std::path::Path, file: &str) -> String {
    std::fs::read_to_string(output.join(file)).unwrap_or_else(|_| panic!("Failed to read {file}"))
}

// --- Basic archive + build ---

#[test]
fn archive_then_build_renders_both_versions() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Test\"\n",
        &[("intro.rdx", "# V1 Intro\n\nOld content.")],
    );

    // Archive v1.0
    let archive = create_archive(tmp.path(), "v1.0").unwrap();
    write_archive(tmp.path(), "v1.0", &archive).unwrap();

    // Update docs
    std::fs::write(
        tmp.path().join("docs/intro.rdx"),
        "# V2 Intro\n\nNew content.",
    )
    .unwrap();

    // Build
    build_site(tmp.path(), &output).unwrap();

    // Current version
    let current = read_output(&output, "intro/index.html");
    assert!(current.contains("V2 Intro"));
    assert!(!current.contains("V1 Intro"));

    // Archived version
    let archived = read_output(&output, "v1.0/intro.html");
    assert!(archived.contains("V1 Intro"));
    assert!(!archived.contains("V2 Intro"));
}

// --- Multi-archive builds ---

#[test]
fn multi_archive_build_renders_all_versions() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Multi\"\n",
        &[("page.rdx", "# V1\n\nFirst.")],
    );

    // Archive v1.0
    let a1 = create_archive(tmp.path(), "v1.0").unwrap();
    write_archive(tmp.path(), "v1.0", &a1).unwrap();

    // Update and archive v2.0
    std::fs::write(tmp.path().join("docs/page.rdx"), "# V2\n\nSecond.").unwrap();
    let a2 = create_archive(tmp.path(), "v2.0").unwrap();
    write_archive(tmp.path(), "v2.0", &a2).unwrap();

    // Update to current
    std::fs::write(tmp.path().join("docs/page.rdx"), "# V3\n\nThird.").unwrap();

    build_site(tmp.path(), &output).unwrap();

    assert!(read_output(&output, "page/index.html").contains("V3"));
    assert!(read_output(&output, "v1.0/page.html").contains("V1"));
    assert!(read_output(&output, "v2.0/page.html").contains("V2"));

    // All three versions appear in the switcher
    let current = read_output(&output, "page/index.html");
    assert!(current.contains("v1.0"));
    assert!(current.contains("v2.0"));
    assert!(current.contains("latest"));
}

// --- Version switcher injection ---

#[test]
fn version_switcher_injected_with_correct_selection() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Switcher\"\n",
        &[("doc.rdx", "# Doc\n\nContent.")],
    );

    let archive = create_archive(tmp.path(), "v1.0").unwrap();
    write_archive(tmp.path(), "v1.0", &archive).unwrap();

    build_site(tmp.path(), &output).unwrap();

    // Current page: "latest" shown in toggle, its link is active
    let current = read_output(&output, "doc/index.html");
    assert!(current.contains("oxidoc-version-switcher"));
    assert!(current.contains(">latest <"));
    assert!(current.contains(r#"href="/" class="oxidoc-version-link active""#));

    // Archived page: "v1.0" shown in toggle, its link is active
    let archived = read_output(&output, "v1.0/doc.html");
    assert!(archived.contains("oxidoc-version-switcher"));
    assert!(archived.contains(">v1.0 <"));
    assert!(archived.contains(r#"class="oxidoc-version-link active" data-version="v1.0""#));
}

#[test]
fn no_version_switcher_without_archives() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"NoArchive\"\n",
        &[("page.rdx", "# Page\n\nContent.")],
    );

    build_site(tmp.path(), &output).unwrap();

    let html = read_output(&output, "page/index.html");
    assert!(!html.contains("oxidoc-version-switcher"));
}

// --- Version-scoped search indices ---

#[test]
fn each_archived_version_has_own_search_index() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Search\"\n",
        &[("page.rdx", "# V1\n\nSearchable content.")],
    );

    let a1 = create_archive(tmp.path(), "v1.0").unwrap();
    write_archive(tmp.path(), "v1.0", &a1).unwrap();

    std::fs::write(tmp.path().join("docs/page.rdx"), "# V2\n\nNew content.").unwrap();
    let a2 = create_archive(tmp.path(), "v2.0").unwrap();
    write_archive(tmp.path(), "v2.0", &a2).unwrap();

    build_site(tmp.path(), &output).unwrap();

    // Each version should have its own search metadata
    assert!(output.join("search-meta.bin").exists());
    assert!(output.join("v1.0/search-meta.bin").exists());
    assert!(output.join("v2.0/search-meta.bin").exists());
}

// --- Archive with routing sections ---

#[test]
fn archive_with_multiple_sections() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let docs = root.join("docs");
    let lib = root.join("lib");
    std::fs::create_dir(&docs).unwrap();
    std::fs::create_dir(&lib).unwrap();
    std::fs::write(
        root.join("oxidoc.toml"),
        r#"[project]
name = "Multi-Section"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Docs", pages = ["intro"] }] },
  { path = "/lib", dir = "lib", groups = [{ group = "Library", pages = ["api"] }] },
]
"#,
    )
    .unwrap();
    std::fs::write(docs.join("intro.rdx"), "# V1 Intro\n\nDocs.").unwrap();
    std::fs::write(lib.join("api.rdx"), "# V1 API\n\nLibrary.").unwrap();

    // Archive
    let archive = create_archive(root, "v1.0").unwrap();
    write_archive(root, "v1.0", &archive).unwrap();

    // Update both sections
    std::fs::write(docs.join("intro.rdx"), "# V2 Intro\n\nNew docs.").unwrap();
    std::fs::write(lib.join("api.rdx"), "# V2 API\n\nNew library.").unwrap();

    let output = root.join("dist");
    build_site(root, &output).unwrap();

    // Current versions
    assert!(read_output(&output, "intro/index.html").contains("V2 Intro"));
    assert!(read_output(&output, "lib/api/index.html").contains("V2 API"));

    // Archived versions
    assert!(read_output(&output, "v1.0/intro.html").contains("V1 Intro"));
    assert!(read_output(&output, "v1.0/lib/api.html").contains("V1 API"));
}

// --- Archive with root pages ---

#[test]
fn archive_with_root_homepage() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let docs = root.join("docs");
    std::fs::create_dir(&docs).unwrap();
    std::fs::write(
        root.join("oxidoc.toml"),
        r#"[project]
name = "WithRoot"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Docs", pages = ["page"] }] },
]

[routing.root]
homepage = "home.rdx"
"#,
    )
    .unwrap();
    std::fs::write(docs.join("page.rdx"), "# Page\n\nContent.").unwrap();
    std::fs::write(root.join("home.rdx"), "# V1 Home\n\nOld homepage.").unwrap();

    let archive = create_archive(root, "v1.0").unwrap();
    assert_eq!(archive.root_pages.len(), 1);
    assert_eq!(archive.root_pages[0].title, "V1 Home");

    write_archive(root, "v1.0", &archive).unwrap();

    // Update homepage
    std::fs::write(root.join("home.rdx"), "# V2 Home\n\nNew homepage.").unwrap();

    let output = root.join("dist");
    build_site(root, &output).unwrap();

    // Current homepage
    assert!(read_output(&output, "index.html").contains("V2 Home"));

    // Archived homepage
    assert!(read_output(&output, "v1.0/index.html").contains("V1 Home"));
}

// --- Archive with landing page layout ---

#[test]
fn archive_with_landing_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let docs = root.join("docs");
    std::fs::create_dir(&docs).unwrap();
    std::fs::write(
        root.join("oxidoc.toml"),
        r#"[project]
name = "Landing"

[routing]
navigation = [
  { path = "/", dir = "docs", groups = [{ group = "Docs", pages = ["page"] }] },
]

[routing.root]
homepage = "home.rdx"
"#,
    )
    .unwrap();
    std::fs::write(docs.join("page.rdx"), "# Doc\n\nContent.").unwrap();
    std::fs::write(
        root.join("home.rdx"),
        "---\nlayout: landing\n---\n# Landing\n\nWelcome!",
    )
    .unwrap();

    let archive = create_archive(root, "v1.0").unwrap();
    write_archive(root, "v1.0", &archive).unwrap();

    let output = root.join("dist");
    build_site(root, &output).unwrap();

    // Archived landing page should have landing class
    let archived_home = read_output(&output, "v1.0/index.html");
    assert!(archived_home.contains("oxidoc-landing"));
    assert!(archived_home.contains("oxidoc-sidebar-landing"));
}

// --- Build without archives produces no version directories ---

#[test]
fn build_without_archives_no_version_dirs() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Clean\"\n",
        &[("page.rdx", "# Page\n\nContent.")],
    );

    build_site(tmp.path(), &output).unwrap();

    // No version subdirectories should exist
    let entries: Vec<_> = std::fs::read_dir(&output)
        .unwrap()
        .filter_map(|e| {
            let e = e.ok()?;
            if e.file_type().ok()?.is_dir() {
                Some(e.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();

    // Should not contain any version directories
    for entry in &entries {
        assert!(
            !entry.starts_with('v'),
            "Unexpected version directory: {entry}"
        );
    }
}

// --- Archived pages use current UI theme ---

#[test]
fn archived_pages_use_current_theme() {
    let (tmp, output) = setup_project(
        "[project]\nname = \"Themed\"\n\n[theme]\nprimary = \"#ff0000\"\n",
        &[("page.rdx", "# Page\n\nContent.")],
    );

    let archive = create_archive(tmp.path(), "v1.0").unwrap();
    write_archive(tmp.path(), "v1.0", &archive).unwrap();

    // Change theme for current build
    std::fs::write(
        tmp.path().join("oxidoc.toml"),
        "[project]\nname = \"Themed\"\n\n[theme]\nprimary = \"#00ff00\"\n",
    )
    .unwrap();

    build_site(tmp.path(), &output).unwrap();

    // Both current and archived should reference the same CSS (current theme)
    let current = read_output(&output, "page/index.html");
    let archived = read_output(&output, "v1.0/page.html");

    // Extract CSS filename from current page
    let css_ref: &str = current
        .split("oxidoc.")
        .nth(1)
        .unwrap()
        .split(".css")
        .next()
        .unwrap();

    // Archived page should reference the same hashed CSS
    assert!(
        archived.contains(&format!("oxidoc.{css_ref}.css")),
        "Archived page should use current theme's CSS"
    );
}
