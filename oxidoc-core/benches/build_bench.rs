use fluxbench::bench;
use fluxbench::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Generate N .rdx files with realistic content
fn generate_rdx_files(docs_dir: &Path, count: usize) -> std::io::Result<()> {
    std::fs::create_dir_all(docs_dir)?;

    for i in 1..=count {
        let filename = format!("{:04}-page.rdx", i);
        let content = format!(
            "# Page {}\n\nThis is a sample documentation page.\n\n## Section A\n\nSome introductory text.\n\n## Section B\n\nMore content with various markdown elements:\n\n- List item 1\n- List item 2\n- List item 3\n\n```rust\nfn example() {{\n    println!(\"Hello from page {}\");\n}}\n```\n\n## Section C\n\nConclusion and summary.",
            i, i
        );
        std::fs::write(docs_dir.join(&filename), content)?;
    }

    Ok(())
}

/// Generate a minimal oxidoc.toml config
fn generate_config(project_root: &Path) -> std::io::Result<()> {
    let config = "[project]\nname = \"Test Documentation\"\ndescription = \"A test documentation site\"\n\n[theme]\nprimary = \"#2563eb\"\ndark_mode = \"system\"\n";
    std::fs::write(project_root.join("oxidoc.toml"), config)?;
    Ok(())
}

/// Setup function that creates a temporary project directory
fn setup_project(page_count: usize) -> (TempDir, PathBuf) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let project_root = tmp_dir.path().to_path_buf();

    generate_config(&project_root).unwrap();
    let docs_dir = project_root.join("docs");
    generate_rdx_files(&docs_dir, page_count).unwrap();

    let output_dir = project_root.join("dist");
    (tmp_dir, output_dir)
}

#[bench(group = "build")]
fn build_100(b: &mut Bencher) {
    b.iter_with_setup(
        || setup_project(100),
        |(tmp_dir, output_dir)| {
            let project_root = tmp_dir.path();
            let _ = oxidoc_core::builder::build_site(project_root, &output_dir);
        },
    );
}

#[bench(group = "build")]
fn build_1000(b: &mut Bencher) {
    b.iter_with_setup(
        || setup_project(1000),
        |(tmp_dir, output_dir)| {
            let project_root = tmp_dir.path();
            let _ = oxidoc_core::builder::build_site(project_root, &output_dir);
        },
    );
}

#[bench(group = "build")]
fn incremental_rebuild(b: &mut Bencher) {
    let (tmp_dir, output_dir) = setup_project(100);
    let project_root = tmp_dir.path();

    // Initial build to populate cache
    let _ = oxidoc_core::builder::build_site(project_root, &output_dir);

    // Now measure the incremental (no-change) rebuild
    b.iter(|| {
        let _ = oxidoc_core::builder::build_site(project_root, &output_dir);
    });
}

#[bench(group = "build")]
fn build_with_simple_config(b: &mut Bencher) {
    let (tmp_dir, output_dir) = setup_project(100);
    let project_root = tmp_dir.path();

    b.iter(|| {
        let _ = oxidoc_core::builder::build_site(project_root, &output_dir);
    });
}

fn main() {
    fluxbench::run().unwrap();
}
