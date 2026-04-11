mod server;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Subcommand)]
enum ArchiveAction {
    /// Create a new archive from current docs
    Create {
        /// Version label (e.g., "v1.0")
        version: String,
    },
    /// List all available archives
    List,
    /// Delete an archive
    Delete {
        /// Version to delete (e.g., "v1.0")
        version: String,
    },
}
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum BuildTarget {
    /// Build web documentation (HTML + Wasm islands)
    Web,
    /// Build PDF output from .rdx files
    Pdf,
    /// Build both web and PDF
    All,
}

#[derive(Parser)]
#[command(
    name = "oxidoc",
    about = "Documentation engine powered by Rust and WebAssembly",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Project root directory (defaults to current directory)
    #[arg(short = 'C', long, global = true)]
    project: Option<PathBuf>,

    /// Enable verbose output (detailed build steps)
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all output except errors (for CI)
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Build the documentation site for production (SSG)
    Build {
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: String,

        /// Build target: web (default), pdf, or all
        #[arg(short, long, default_value = "web")]
        target: BuildTarget,

        /// (PDF only) Draw debug borders on every element to visualize layout boxes
        #[arg(long)]
        debug_boxes: bool,
    },
    /// Start the development server with hot module replacement
    Dev {
        /// Port to serve on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Initialize a new Oxidoc project
    Init {
        /// Project directory name (defaults to current directory)
        name: Option<String>,

        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Skip confirmation prompts (auto-confirm)
        #[arg(short, long)]
        yes: bool,
    },
    /// Manage versioned documentation archives
    Archive {
        #[command(subcommand)]
        action: ArchiveAction,
    },
    /// Remove build artifacts (.oxidoc-dev/ and dist/)
    Clean,
    /// Update to the latest version
    Update {
        /// Install the latest prerelease instead of stable
        #[arg(long)]
        pre: bool,
    },
    /// Switch to a specific version
    SetVersion {
        /// Version to switch to (e.g., v0.1.0, v0.1.0-beta.6)
        version: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let log_level = if cli.quiet {
        tracing::Level::ERROR
    } else if cli.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"))
        .canonicalize()
        .expect("cannot resolve project path — does the directory exist?");

    let result = match cli.command {
        Command::Build {
            output,
            target,
            debug_boxes,
        } => run_build(&project_root, &output, target, debug_boxes),
        Command::Dev { port } => run_dev(&project_root, port),
        Command::Init { name, force, yes } => {
            let target = match name {
                Some(ref n) => project_root.join(n),
                None => project_root.clone(),
            };
            run_init(&target, name.as_deref(), force, yes)
        }
        Command::Archive { action } => match action {
            ArchiveAction::Create { version } => run_archive(&project_root, &version),
            ArchiveAction::List => run_archive_list(&project_root),
            ArchiveAction::Delete { version } => run_archive_delete(&project_root, &version),
        },
        Command::Clean => run_clean(&project_root),
        Command::Update { pre } => run_self_update(pre),
        Command::SetVersion { version } => run_set_version(&version),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let code = e
                .downcast_ref::<oxidoc_core::error::OxidocError>()
                .is_some_and(|oe| oe.is_config_error());
            eprintln!("{e:?}");
            ExitCode::from(if code { 2 } else { 1 })
        }
    }
}

/// Write Wasm assets to the output directory.
///
/// In development (inside the oxidoc workspace), builds from source to get the latest code.
/// Otherwise, writes the pre-compiled bundled assets — no Rust toolchain required.
fn write_wasm_assets(output_dir: &std::path::Path) {
    if is_oxidoc_workspace() {
        match oxidoc_core::wasm::build_wasm(output_dir) {
            Ok(()) => return,
            Err(e) => {
                tracing::warn!("Wasm source build failed ({e}), falling back to bundled assets");
            }
        }
    }

    if let Err(e) = oxidoc_core::wasm::write_bundled_wasm(output_dir, &BUNDLED_WASM) {
        tracing::warn!("Failed to write bundled wasm assets: {e}");
    }
}

/// Check if we're running inside the oxidoc development workspace.
fn is_oxidoc_workspace() -> bool {
    let output = std::process::Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output();

    let Ok(output) = output else { return false };
    if !output.status.success() {
        return false;
    }

    let manifest = String::from_utf8_lossy(&output.stdout);
    let workspace_root = std::path::Path::new(manifest.trim()).parent();
    match workspace_root {
        Some(root) => root.join("oxidoc-registry").is_dir(),
        None => false,
    }
}

/// Bundled sentence embedding model for hybrid search.
const BUNDLED_SEARCH_MODEL: &[u8] = include_bytes!("../assets/models/bge-micro-v2.gguf");

/// Pre-compiled Wasm assets (JS glue + Wasm binaries) built during CI.
const BUNDLED_WASM: oxidoc_core::wasm::BundledWasm = oxidoc_core::wasm::BundledWasm {
    registry_js: include_bytes!("../assets/wasm/oxidoc_registry.js"),
    registry_wasm: include_bytes!("../assets/wasm/oxidoc_registry_bg.wasm"),
    openapi_js: include_bytes!("../assets/wasm/oxidoc_openapi.js"),
    openapi_wasm: include_bytes!("../assets/wasm/oxidoc_openapi_bg.wasm"),
    search_js: include_bytes!("../assets/wasm/oxidoc_search.js"),
    search_wasm: include_bytes!("../assets/wasm/oxidoc_search_bg.wasm"),
};

fn run_build(
    project_root: &std::path::Path,
    output: &str,
    target: BuildTarget,
    debug_boxes: bool,
) -> miette::Result<()> {
    match target {
        BuildTarget::Web => run_build_web(project_root, output),
        BuildTarget::Pdf => run_build_pdf(project_root, output, debug_boxes),
        BuildTarget::All => {
            run_build_web(project_root, output)?;
            run_build_pdf(project_root, output, debug_boxes)?;
            Ok(())
        }
    }
}

fn run_build_web(project_root: &std::path::Path, output: &str) -> miette::Result<()> {
    let output_dir = project_root.join(output);
    tracing::info!("Building site to {}/", output_dir.display());

    write_wasm_assets(&output_dir);

    let start = std::time::Instant::now();
    let result = oxidoc_core::builder::build_site_with_model(
        project_root,
        &output_dir,
        Some(BUNDLED_SEARCH_MODEL),
    )?;
    tracing::info!(
        pages = result.pages_rendered,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "Build complete"
    );
    Ok(())
}

fn run_build_pdf(
    project_root: &std::path::Path,
    output: &str,
    debug_boxes: bool,
) -> miette::Result<()> {
    let output_dir = project_root.join(output);
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| miette::miette!("Failed to create output directory: {e}"))?;

    tracing::info!("Building PDF to {}/", output_dir.display());
    let start = std::time::Instant::now();

    // Discover .rdx files
    let docs_dir = project_root.join("docs");
    if !docs_dir.is_dir() {
        miette::bail!("No docs/ directory found in {}", project_root.display());
    }

    let rdx_files = discover_rdx_files(&docs_dir);
    if rdx_files.is_empty() {
        miette::bail!("No .rdx files found in {}", docs_dir.display());
    }

    let mut config =
        oxidoc_print::config::PrintConfig::default_with_root(project_root.to_path_buf());
    config.debug_boxes = debug_boxes;

    // Concatenate all .rdx files into a single document (simple single-file approach)
    // Multi-file book assembly comes in a later batch via [book] config
    let mut combined_source = String::new();
    for path in &rdx_files {
        let content = std::fs::read_to_string(path)
            .map_err(|e| miette::miette!("Failed to read {}: {e}", path.display()))?;
        if !combined_source.is_empty() {
            combined_source.push_str("\n\n---\n\n");
        }
        // Strip frontmatter from non-first files to avoid parser issues
        let content = if !combined_source.is_empty() {
            strip_frontmatter(&content)
        } else {
            content.as_str().to_string()
        };
        combined_source.push_str(&content);
    }

    let pdf_bytes = oxidoc_print::render_file_to_pdf(&combined_source, &config)
        .map_err(|e| miette::miette!("PDF rendering failed: {e}"))?;

    let pdf_path = output_dir.join("book.pdf");
    std::fs::write(&pdf_path, &pdf_bytes)
        .map_err(|e| miette::miette!("Failed to write {}: {e}", pdf_path.display()))?;

    tracing::info!(
        pages = rdx_files.len(),
        bytes = pdf_bytes.len(),
        elapsed_ms = start.elapsed().as_millis() as u64,
        path = %pdf_path.display(),
        "PDF build complete"
    );
    Ok(())
}

fn discover_rdx_files(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(discover_rdx_files(&path));
            } else if path.extension().is_some_and(|e| e == "rdx") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn strip_frontmatter(content: &str) -> String {
    if !content.starts_with("---") {
        return content.to_string();
    }
    // Find the closing ---
    if let Some(end) = content[3..].find("\n---") {
        let after = end + 3 + 4; // skip past \n---
        if after < content.len() {
            return content[after..].to_string();
        }
    }
    content.to_string()
}

fn run_dev(project_root: &std::path::Path, port: u16) -> miette::Result<()> {
    // Clean dev output before starting
    let dev_dir = project_root.join(".oxidoc-dev");
    if dev_dir.exists() {
        std::fs::remove_dir_all(&dev_dir)
            .map_err(|e| miette::miette!("Failed to clean .oxidoc-dev/: {e}"))?;
        tracing::debug!("Cleaned .oxidoc-dev/");
    }

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| miette::miette!("Failed to create async runtime: {e}"))?;
    rt.block_on(server::run_dev_server(project_root.to_path_buf(), port))
}

fn run_archive(project_root: &std::path::Path, version: &str) -> miette::Result<()> {
    let start = std::time::Instant::now();
    tracing::info!(version = %version, "Archiving current docs");

    let archive = oxidoc_core::archive::create_archive(project_root, version)?;
    oxidoc_core::archive::write_archive(project_root, version, &archive)?;

    tracing::info!(
        version = %version,
        pages = archive.pages.len(),
        elapsed_ms = start.elapsed().as_millis() as u64,
        "Archive complete"
    );
    Ok(())
}

fn run_archive_list(project_root: &std::path::Path) -> miette::Result<()> {
    let versions = oxidoc_core::archive::discover_archives(project_root);
    if versions.is_empty() {
        tracing::info!("No archives found");
    } else {
        for version in &versions {
            println!("{version}");
        }
        tracing::info!(count = versions.len(), "Archives listed");
    }
    Ok(())
}

fn run_archive_delete(project_root: &std::path::Path, version: &str) -> miette::Result<()> {
    let archive_path = project_root
        .join("archives")
        .join(format!("{version}.rdx.archive"));
    if !archive_path.exists() {
        miette::bail!("Archive '{version}' not found");
    }
    std::fs::remove_file(&archive_path)
        .map_err(|e| miette::miette!("Failed to delete archive: {e}"))?;
    tracing::info!(version = %version, "Archive deleted");
    Ok(())
}

fn run_clean(project_root: &std::path::Path) -> miette::Result<()> {
    let mut cleaned = false;
    for dir_name in [".oxidoc-dev", "dist"] {
        let dir = project_root.join(dir_name);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .map_err(|e| miette::miette!("Failed to remove {dir_name}/: {e}"))?;
            tracing::info!("Removed {dir_name}/");
            cleaned = true;
        }
    }
    if !cleaned {
        tracing::info!("Nothing to clean");
    }
    Ok(())
}

fn run_init(
    target: &std::path::Path,
    name: Option<&str>,
    force: bool,
    yes: bool,
) -> miette::Result<()> {
    std::fs::create_dir_all(target)
        .map_err(|e| miette::miette!("Failed to create project directory: {e}"))?;

    let config_path = target.join("oxidoc.toml");
    if config_path.exists() && !force {
        miette::bail!(
            "oxidoc.toml already exists in {}. Use --force to overwrite.",
            target.display()
        );
    }

    if config_path.exists() && force && !yes {
        eprint!("Overwrite existing project in {}? [y/N] ", target.display());
        let _ = std::io::stderr().flush();
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| miette::miette!("Failed to read input: {e}"))?;
        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            miette::bail!("Aborted");
        }
    }

    let project_name = name
        .map(|n| {
            std::path::Path::new(n)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or(n)
        })
        .unwrap_or("Oxidoc");

    // Create directories
    let docs_dir = target.join("docs");
    let lib_dir = target.join("lib");
    let assets_dir = target.join("assets");
    let deploy_dir = docs_dir.join("deployment");
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| miette::miette!("Failed to create docs/: {e}"))?;
    let guides_dir = docs_dir.join("guides");
    std::fs::create_dir_all(&deploy_dir)
        .map_err(|e| miette::miette!("Failed to create docs/deployment/: {e}"))?;
    std::fs::create_dir_all(&guides_dir)
        .map_err(|e| miette::miette!("Failed to create docs/guides/: {e}"))?;
    std::fs::create_dir_all(&lib_dir).map_err(|e| miette::miette!("Failed to create lib/: {e}"))?;
    std::fs::create_dir_all(&assets_dir)
        .map_err(|e| miette::miette!("Failed to create assets/: {e}"))?;

    // Write files from embedded templates
    write_file(
        &assets_dir.join("logo.svg"),
        include_str!("../assets/logo.svg"),
    )?;
    write_file(
        &target.join(".gitignore"),
        "# Oxidoc build artifacts\n.oxidoc-dev/\ndist/\n",
    )?;
    write_file(
        &config_path,
        &include_str!("../assets/templates/oxidoc.toml.tmpl")
            .replace("{project_name}", project_name),
    )?;
    write_file(
        &target.join("home.rdx"),
        include_str!("../assets/templates/home.rdx"),
    )?;
    write_file(
        &docs_dir.join("quickstart.rdx"),
        include_str!("../assets/templates/quickstart.rdx"),
    )?;
    write_file(
        &docs_dir.join("api-reference.rdx"),
        include_str!("../assets/templates/api-reference.rdx"),
    )?;
    write_file(
        &docs_dir.join("advanced.rdx"),
        include_str!("../assets/templates/advanced.rdx"),
    )?;
    write_file(
        &docs_dir.join("deployment.rdx"),
        include_str!("../assets/templates/deployment.rdx"),
    )?;
    write_file(
        &deploy_dir.join("github-pages.rdx"),
        include_str!("../assets/templates/deployment-github-pages.rdx"),
    )?;
    write_file(
        &deploy_dir.join("vercel.rdx"),
        include_str!("../assets/templates/deployment-vercel.rdx"),
    )?;
    write_file(
        &deploy_dir.join("netlify.rdx"),
        include_str!("../assets/templates/deployment-netlify.rdx"),
    )?;
    write_file(
        &guides_dir.join("styling.rdx"),
        include_str!("../assets/templates/guides-styling.rdx"),
    )?;
    write_file(
        &guides_dir.join("animations.rdx"),
        include_str!("../assets/templates/guides-animations.rdx"),
    )?;
    write_file(
        &guides_dir.join("internationalization.rdx"),
        include_str!("../assets/templates/guides-internationalization.rdx"),
    )?;
    write_file(
        &lib_dir.join("index.rdx"),
        include_str!("../assets/templates/lib-index.rdx"),
    )?;
    write_file(
        &lib_dir.join("manual-api.rdx"),
        include_str!("../assets/templates/lib-manual-api.rdx"),
    )?;
    write_file(
        &lib_dir.join("library-example.rdx"),
        include_str!("../assets/templates/lib-library-example.rdx"),
    )?;
    write_file(
        &target.join("openapi.yaml"),
        &include_str!("../assets/templates/openapi.yaml").replace("{project_name}", project_name),
    )?;

    // Initialize git repo if not already inside one
    let in_git = std::process::Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(target)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !in_git {
        let _ = std::process::Command::new("git")
            .arg("init")
            .current_dir(target)
            .output();
    }

    if let Some(n) = name {
        eprintln!("Initialized Oxidoc project in {n}/");
    } else {
        eprintln!("Initialized Oxidoc project");
    }
    eprintln!();
    eprintln!("  oxidoc.toml              — site configuration");
    eprintln!("  openapi.yaml             — sample API spec (replace with your own)");
    eprintln!("  assets/logo.svg          — default logo (replace with your own)");
    eprintln!("  docs/                    — main documentation pages");
    eprintln!("  docs/deployment.rdx      — deployment guide (GitHub Pages, Vercel, Netlify)");
    eprintln!("  lib/                     — library API documentation (separate section)");
    eprintln!("  lib/library-example.rdx  — example: documenting a library API");
    eprintln!();
    if let Some(n) = name {
        eprintln!("Get started:");
        eprintln!("  cd {n} && oxidoc dev");
    } else {
        eprintln!("Get started:");
        eprintln!("  oxidoc dev");
    }

    Ok(())
}

fn write_file(path: &std::path::Path, content: &str) -> miette::Result<()> {
    std::fs::write(path, content)
        .map_err(|e| miette::miette!("Failed to write {}: {e}", path.display()))
}

const INSTALL_SCRIPT_URL: &str = "https://oxidoc.dev/install.sh";

fn run_self_update(pre: bool) -> miette::Result<()> {
    let mut args = vec!["sh", "-s", "--"];
    if pre {
        args.push("--pre");
    }

    run_install_script(&args)
}

fn run_set_version(version: &str) -> miette::Result<()> {
    let tag = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    };

    run_install_script(&["sh", "-s", "--", "--version", &tag])
}

fn run_install_script(sh_args: &[&str]) -> miette::Result<()> {
    // Determine which downloader is available
    let (dl_cmd, dl_args): (&str, &[&str]) = if which("curl") {
        ("curl", &["-fsSL", INSTALL_SCRIPT_URL])
    } else if which("wget") {
        ("wget", &["-qO-", INSTALL_SCRIPT_URL])
    } else {
        miette::bail!("curl or wget is required for self-update");
    };

    // Pipe: curl/wget -> sh -s -- [args]
    let downloader = std::process::Command::new(dl_cmd)
        .args(dl_args)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| miette::miette!("Failed to start {dl_cmd}: {e}"))?;

    let status = std::process::Command::new(sh_args[0])
        .args(&sh_args[1..])
        .stdin(downloader.stdout.unwrap())
        .status()
        .map_err(|e| miette::miette!("Failed to run install script: {e}"))?;

    if !status.success() {
        miette::bail!("Install script failed");
    }

    Ok(())
}

fn which(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
