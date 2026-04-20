/// Integration tests for the Oxidoc CLI.
/// Uses std::process::Command to invoke the compiled binary and verify outputs.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the compiled `oxidoc` binary.
fn get_oxidoc_binary() -> PathBuf {
    // In tests, we can use env!("CARGO_BIN_EXE_oxidoc") or find the binary in target/debug/
    let exe = if cfg!(windows) {
        "oxidoc.exe"
    } else {
        "oxidoc"
    };

    // Try to find it in the standard Cargo output location
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from oxidoc-cli/
    path.push("target/debug");
    path.push(exe);

    if !path.exists() {
        // Try release build
        let mut release_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        release_path.pop();
        release_path.push("target/release");
        release_path.push(exe);
        if release_path.exists() {
            return release_path;
        }
    }

    path
}

/// Run oxidoc with the given arguments in a project directory.
fn run_oxidoc(project_dir: &Path, args: &[&str]) -> std::process::Output {
    let binary = get_oxidoc_binary();
    let mut cmd = Command::new(&binary);
    cmd.current_dir(project_dir);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.output().expect("Failed to execute oxidoc binary")
}

/// Create a minimal oxidoc.toml file.
fn create_config(dir: &Path, project_name: &str) -> std::io::Result<()> {
    let config = format!(
        r#"[project]
name = "{}"
"#,
        project_name
    );
    fs::write(dir.join("oxidoc.toml"), config)?;
    Ok(())
}

/// Create a sample RDX file.
fn create_rdx_file(docs_dir: &Path, name: &str, content: &str) -> std::io::Result<()> {
    fs::create_dir_all(docs_dir)?;
    fs::write(docs_dir.join(format!("{}.rdx", name)), content)?;
    Ok(())
}

#[test]
fn test_init_creates_scaffold() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();

    let output = run_oxidoc(project_dir, &["init"]);

    // Should succeed
    assert!(
        output.status.success(),
        "init should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that core files and directories were created
    assert!(
        project_dir.join("oxidoc.toml").exists(),
        "oxidoc.toml should be created"
    );
    assert!(
        project_dir.join("home.rdx").exists(),
        "home.rdx should be created"
    );
    assert!(
        project_dir.join("docs").is_dir(),
        "docs/ directory should be created"
    );
    assert!(
        project_dir.join("docs/quickstart.rdx").exists(),
        "docs/quickstart.rdx should be created"
    );
    assert!(
        project_dir.join("docs/deployment").is_dir(),
        "docs/deployment/ directory should be created"
    );
    assert!(
        project_dir.join("docs/guides").is_dir(),
        "docs/guides/ directory should be created"
    );
    assert!(
        project_dir.join("lib").is_dir(),
        "lib/ directory should be created"
    );
    assert!(
        project_dir.join("assets/logo.svg").exists(),
        "assets/logo.svg should be created"
    );

    // Verify config content
    let config_content =
        fs::read_to_string(project_dir.join("oxidoc.toml")).expect("Failed to read oxidoc.toml");
    assert!(
        config_content.contains("[project]"),
        "Config should have [project] section"
    );
    assert!(
        config_content.contains("name ="),
        "Config should have project name"
    );
}

#[test]
fn test_build_with_missing_config() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();

    // No oxidoc.toml file
    let output = run_oxidoc(project_dir, &["build"]);

    // Should fail with exit code 2 (config error)
    assert!(!output.status.success(), "build should fail without config");
    let status_code = output.status.code().unwrap_or(-1);
    assert_eq!(status_code, 2, "Exit code should be 2 for config error");
}

#[test]
fn test_build_successful() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");
    let dist_dir = project_dir.join("dist");

    // Create config and RDX file
    create_config(project_dir, "Test Docs").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "# Welcome\n\nThis is a test page.")
        .expect("Failed to create RDX file");

    // Run build
    let output = run_oxidoc(project_dir, &["build"]);

    // Should succeed
    assert!(
        output.status.success(),
        "build should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check output directory was created
    assert!(dist_dir.exists(), "dist/ directory should be created");

    // Check that HTML was generated
    assert!(
        dist_dir.join("index.html").exists(),
        "index.html should be generated"
    );

    // Check that CSS and JS assets were generated
    let css_files: Vec<_> = fs::read_dir(&dist_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "css"))
        .collect();
    assert!(!css_files.is_empty(), "CSS assets should be generated");

    let js_files: Vec<_> = fs::read_dir(&dist_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "js"))
        .collect();
    assert!(!js_files.is_empty(), "JS assets should be generated");
}

#[test]
fn test_build_with_custom_output_dir() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");
    let custom_output = project_dir.join("custom_output");

    // Create config and RDX file
    create_config(project_dir, "Test Docs").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "# Welcome").expect("Failed to create RDX file");

    // Run build with custom output dir
    let output = run_oxidoc(project_dir, &["build", "--output", "custom_output"]);

    assert!(
        output.status.success(),
        "build should succeed with custom output"
    );
    assert!(
        custom_output.join("index.html").exists(),
        "HTML should be generated in custom output dir"
    );
}

#[test]
fn test_build_with_multiple_pages() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");
    let dist_dir = project_dir.join("dist");

    // Create config
    create_config(project_dir, "Multi-page Docs").expect("Failed to create config");

    // Create multiple RDX files
    create_rdx_file(&docs_dir, "index", "# Home").expect("Failed to create index");
    create_rdx_file(&docs_dir, "guide", "# Getting Started\n\nHere's a guide.")
        .expect("Failed to create guide");
    create_rdx_file(&docs_dir, "api", "# API Reference\n\nAPI docs.")
        .expect("Failed to create api");

    // Run build
    let output = run_oxidoc(project_dir, &["build"]);

    assert!(
        output.status.success(),
        "build should succeed with multiple pages"
    );

    // All pages should be generated
    assert!(
        dist_dir.join("index.html").exists(),
        "index.html should exist"
    );
    assert!(
        dist_dir.join("guide").join("index.html").exists(),
        "guide/index.html should exist"
    );
    assert!(
        dist_dir.join("api").join("index.html").exists(),
        "api/index.html should exist"
    );
}

#[test]
fn test_build_help_output() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let output = run_oxidoc(temp.path(), &["build", "--help"]);

    assert!(output.status.success(), "help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Build the documentation site"),
        "Help should contain command description"
    );
    assert!(
        stdout.contains("--output") || stdout.contains("-o"),
        "Help should mention output option"
    );
}

#[test]
fn test_invalid_subcommand() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let output = run_oxidoc(temp.path(), &["invalid_command"]);

    assert!(!output.status.success(), "Invalid command should fail");
}

#[test]
fn test_build_with_nested_pages() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");
    let dist_dir = project_dir.join("dist");

    // Create config
    create_config(project_dir, "Nested Docs").expect("Failed to create config");

    // Create pages in nested directories
    fs::create_dir_all(docs_dir.join("guides")).expect("Failed to create guides dir");
    fs::create_dir_all(docs_dir.join("api")).expect("Failed to create api dir");

    create_rdx_file(&docs_dir, "index", "# Home").expect("Failed to create index");
    create_rdx_file(&docs_dir.join("guides"), "intro", "# Getting Started")
        .expect("Failed to create guides/intro");
    create_rdx_file(&docs_dir.join("api"), "overview", "# API Overview")
        .expect("Failed to create api/overview");

    // Run build
    let output = run_oxidoc(project_dir, &["build"]);

    assert!(
        output.status.success(),
        "build should succeed with nested structure"
    );

    // Check that index exists
    assert!(
        dist_dir.join("index.html").exists(),
        "index.html should exist"
    );
}

#[test]
fn test_build_empty_file() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");
    let dist_dir = project_dir.join("dist");

    // Create config with empty RDX file
    create_config(project_dir, "Empty File Test").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "").expect("Failed to create empty RDX");

    // Run build - should still succeed
    let output = run_oxidoc(project_dir, &["build"]);

    assert!(
        output.status.success(),
        "build should succeed with empty files"
    );
    assert!(
        dist_dir.join("index.html").exists(),
        "HTML should be generated even for empty RDX"
    );
}

#[test]
#[ignore] // Marked as ignore per requirements (server-related test)
fn test_dev_server_starts() {
    // This test would require more complex setup (background process management)
    // For now, we mark it as ignored as requested in requirements
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");

    create_config(project_dir, "Dev Server Test").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "# Welcome").expect("Failed to create RDX");

    // In a real implementation, you'd spawn the dev server and verify it listens
    // let output = run_oxidoc(project_dir, &["dev", "--port", "3001"]);
    // assert!(output.status.success());
}

#[test]
fn test_verbose_output() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");

    create_config(project_dir, "Verbose Test").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "# Test").expect("Failed to create RDX");

    let output = run_oxidoc(project_dir, &["--verbose", "build"]);

    assert!(
        output.status.success(),
        "build should succeed with verbose flag"
    );
    // Verbose output would be in stderr or stdout
}

#[test]
fn test_quiet_output() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp.path();
    let docs_dir = project_dir.join("docs");

    create_config(project_dir, "Quiet Test").expect("Failed to create config");
    create_rdx_file(&docs_dir, "index", "# Test").expect("Failed to create RDX");

    let output = run_oxidoc(project_dir, &["--quiet", "build"]);

    assert!(
        output.status.success(),
        "build should succeed with quiet flag"
    );
}
