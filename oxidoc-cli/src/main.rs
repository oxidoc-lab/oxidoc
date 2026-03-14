mod server;

use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "oxidoc",
    about = "Blazing-fast documentation engine powered by Rust and WebAssembly"
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
    /// Remove build artifacts (.oxidoc-dev/ and dist/)
    Clean,
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
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let result = match cli.command {
        Command::Build { output } => run_build(&project_root, &output),
        Command::Dev { port } => run_dev(&project_root, port),
        Command::Init { name, force, yes } => {
            let target = match name {
                Some(ref n) => project_root.join(n),
                None => project_root.clone(),
            };
            run_init(&target, name.as_deref(), force, yes)
        }
        Command::Clean => run_clean(&project_root),
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

fn build_wasm_once(output_dir: &std::path::Path) {
    match oxidoc_core::wasm::build_wasm(output_dir) {
        Ok(()) => {}
        Err(e) => {
            let msg = format!("{e}");
            if msg.contains("locate") {
                tracing::debug!("Skipping wasm build (not in oxidoc workspace)");
            } else {
                tracing::warn!("Wasm build failed: {e}");
            }
        }
    }
}

/// Bundled sentence embedding model for hybrid search.
const BUNDLED_SEARCH_MODEL: &[u8] = include_bytes!("../assets/models/bge-micro-v2.gguf");

fn run_build(project_root: &std::path::Path, output: &str) -> miette::Result<()> {
    let output_dir = project_root.join(output);
    tracing::info!("Building site to {}/", output_dir.display());

    build_wasm_once(&output_dir);

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
    std::fs::create_dir_all(&deploy_dir)
        .map_err(|e| miette::miette!("Failed to create docs/deployment/: {e}"))?;
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
