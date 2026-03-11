mod server;

use clap::{Parser, Subcommand};
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
    Init,
}

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let project_root = cli
        .project
        .unwrap_or_else(|| std::env::current_dir().expect("cannot determine current directory"));

    let result = match cli.command {
        Command::Build { output } => run_build(&project_root, &output),
        Command::Dev { port } => run_dev(&project_root, port),
        Command::Init => run_init(&project_root),
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

fn run_build(project_root: &std::path::Path, output: &str) -> miette::Result<()> {
    let output_dir = project_root.join(output);
    tracing::info!("Building site to {}/", output_dir.display());

    let start = std::time::Instant::now();
    let result = oxidoc_core::builder::build_site(project_root, &output_dir)?;
    tracing::info!(
        pages = result.pages_rendered,
        elapsed_ms = start.elapsed().as_millis() as u64,
        "Build complete"
    );
    Ok(())
}

fn run_dev(project_root: &std::path::Path, port: u16) -> miette::Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| miette::miette!("Failed to create async runtime: {e}"))?;
    rt.block_on(server::run_dev_server(project_root.to_path_buf(), port))
}

fn run_init(project_root: &std::path::Path) -> miette::Result<()> {
    let config_path = project_root.join("oxidoc.toml");
    if config_path.exists() {
        miette::bail!("oxidoc.toml already exists in this directory");
    }

    let docs_dir = project_root.join("docs");
    std::fs::create_dir_all(&docs_dir)
        .map_err(|e| miette::miette!("Failed to create docs/ directory: {e}"))?;

    std::fs::write(
        &config_path,
        r##"[project]
name = "My Documentation"

[theme]
primary = "#3b82f6"
dark_mode = "system"
"##,
    )
    .map_err(|e| miette::miette!("Failed to write oxidoc.toml: {e}"))?;

    std::fs::write(
        docs_dir.join("intro.rdx"),
        r#"# Welcome

Welcome to your new documentation site, powered by **Oxidoc**.

## Getting Started

Edit this file at `docs/intro.rdx` to get started.

<Callout type="info">
This is an interactive island component. It will be hydrated by WebAssembly in the browser.
</Callout>
"#,
    )
    .map_err(|e| miette::miette!("Failed to write intro.rdx: {e}"))?;

    tracing::info!("Initialized new Oxidoc project");
    tracing::info!("  Created oxidoc.toml");
    tracing::info!("  Created docs/intro.rdx");
    tracing::info!("  Run `oxidoc build` to generate your site");

    Ok(())
}
